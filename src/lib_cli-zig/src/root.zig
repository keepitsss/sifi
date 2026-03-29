const std = @import("std");
const assert = std.debug.assert;

// TODO: dealloc messages

pub const CliContext = struct {
    args: [][:0]u8,
    allocator: std.mem.Allocator,
    cursor: usize,
    diagnostics: std.ArrayList(Diagnostic),
    documenataion_store: DocumentationStore,
    output: *std.io.Writer,

    const DiagnosticKind = enum {
        err,
        help,
        warn,
    };
    const Diagnostic = struct {
        kind: DiagnosticKind,
        message: []const u8,
    };

    pub fn init(allocator: std.mem.Allocator, program_docs: Documentation, output: *std.io.Writer) !CliContext {
        return .{
            .args = try std.process.argsAlloc(allocator),
            .cursor = 1,
            .diagnostics = std.ArrayList(Diagnostic).empty,
            .documenataion_store = .{
                .top_level = program_docs,
                .other = std.ArrayList(Documentation).empty,
            },
            .output = output,
            .allocator = allocator,
        };
    }
    pub fn deinit(self: *CliContext) void {
        std.process.argsFree(self.allocator, self.args);
        assert(self.diagnostics.items.len == 0);
        self.diagnostics.deinit(self.allocator);
    }

    pub fn parse(self: *CliContext, options: []const *Option) !void {
        var help_flag = BoolFlag.new(.{
            .names = .{ .main = "--help", .short = "-h" },
            .description = "print help",
        });

        for (options) |option| {
            try option.add_documentation(option, self);
        }
        try help_flag.vtable.add_documentation(&help_flag.vtable, self);

        defer self.emit_diagnostics() catch {};
        var progress_happened = true;
        outer: while (progress_happened) {
            progress_happened = false;
            for (options) |option| {
                switch (try option.try_parse_self(option, self)) {
                    .continue_no_progress => {},
                    .continue_with_progress => {
                        progress_happened = true;
                    },
                    .stop => {
                        break :outer;
                    },
                }
            }
            switch (try help_flag.vtable.try_parse_self(&help_flag.vtable, self)) {
                .continue_no_progress => {},
                .continue_with_progress => {
                    try self.documenataion_store.print(self.output);
                    return error.HelpRequested;
                },
                .stop => unreachable,
            }
        }

        for (options) |option| {
            try option.finalize(option, self);
        }
    }
    fn emit_diagnostics(self: *CliContext) !void {
        for (self.diagnostics.items) |diagnostic| {
            switch (diagnostic.kind) {
                .err => {
                    try self.output.print("\x1b[31mERROR\x1b[0m: {s}\n", .{diagnostic.message});
                },
                .help => {
                    try self.output.print("\x1b[34mHELP\x1b[0m: {s}\n", .{diagnostic.message});
                },
                .warn => {
                    try self.output.print("\x1b[33mWARNING\x1b[0m: {s}\n", .{diagnostic.message});
                },
            }
        }
        try self.output.flush();
        self.diagnostics.clearRetainingCapacity();
    }
};

pub const Documentation = struct {
    // TODO: Maybe make it enum or smth like that
    section: ?[]const u8 = null,
    names: struct {
        main: []const u8,
        short: ?[]const u8 = null,
        aliases: []const []const u8 = &.{},
    },
    description: []const u8,
};

const DocumentationStore = struct {
    top_level: Documentation,
    other: std.ArrayList(Documentation),

    fn lessThan(_: void, left: Documentation, right: Documentation) bool {
        if (left.section == null) return true;
        if (right.section == null) return false;
        return std.mem.order(u8, left.section.?, right.section.?) == .lt;
    }
    pub fn print(self: *const DocumentationStore, writer: *std.io.Writer) !void {
        try writer.print("\x1b[1m{s}\x1b[0m - {s}\n", .{
            self.top_level.names.main,
            self.top_level.description,
        });
        std.sort.block(Documentation, self.other.items, {}, lessThan);
        var max_short_name_len: usize = 0;
        var max_main_name_len: usize = 0;
        for (self.other.items) |doc| {
            if (doc.names.short) |short| {
                max_short_name_len = @max(max_short_name_len + 1, short.len);
            }
            max_main_name_len = @max(max_main_name_len, doc.names.main.len);
        }
        var prev_section: ?[]const u8 = null;
        for (self.other.items) |doc| {
            if (doc.section != null and (prev_section == null or !std.mem.eql(u8, doc.section.?, prev_section.?))) {
                if (doc.section) |section| {
                    try writer.print("\x1b[1;4m{s}s:\x1b[0m\n", .{section});
                }
                prev_section = doc.section;
            }
            try writer.print("  \x1b[1m", .{});

            if (doc.names.short) |short| {
                try writer.print("{s},", .{short});
                try writer.splatByteAll(' ', max_short_name_len - (short.len + 1));
            } else {
                try writer.splatByteAll(' ', max_short_name_len);
            }

            try writer.print(" {s}\x1b[0m  ", .{doc.names.main});
            try writer.splatByteAll(' ', max_main_name_len - doc.names.main.len);

            try writer.print("{s}", .{doc.description});

            if (doc.names.aliases.len > 0) {
                try writer.print(" [aliases: ", .{});
                for (doc.names.aliases, 0..) |alias, i| {
                    if (i > 0) try writer.print(", ", .{});
                    try writer.print("{s}", .{alias});
                }
                try writer.print("]", .{});
            }

            try writer.print("\n", .{});
        }
        try writer.flush();
    }
};

pub const Option = struct {
    add_documentation: *const fn (*Option, *CliContext) std.mem.Allocator.Error!void,
    try_parse_self: *const fn (*Option, *CliContext) anyerror!ParsingResult,
    finalize: *const fn (*Option, *CliContext) anyerror!void,

    const ParsingResult = enum {
        continue_no_progress,
        continue_with_progress,
        stop,
    };
};

pub const BoolFlag = struct {
    occurances: usize = 0,
    documentation: Documentation,
    vtable: Option = .{
        .add_documentation = add_documentation,
        .try_parse_self = try_parse,
        .finalize = finalize,
    },

    pub fn new(documentation: Documentation) BoolFlag {
        var doc = documentation;
        if (doc.section) |section| {
            assert(std.mem.eql(u8, section, "flag"));
        }
        if (doc.section == null) {
            doc.section = "flag";
        }
        return .{ .documentation = doc };
    }

    pub fn present(self: *const BoolFlag) bool {
        return self.occurances > 0;
    }

    fn add_documentation(ptr: *Option, ctx: *CliContext) std.mem.Allocator.Error!void {
        const self: *BoolFlag = @fieldParentPtr("vtable", ptr);
        try ctx.documenataion_store.other.append(ctx.allocator, self.documentation);
    }
    fn try_parse(ptr: *Option, ctx: *CliContext) anyerror!Option.ParsingResult {
        const self: *BoolFlag = @fieldParentPtr("vtable", ptr);
        if (ctx.cursor >= ctx.args.len) return .continue_no_progress;
        if (std.mem.eql(u8, ctx.args[ctx.cursor], self.documentation.names.main)) {
            try self.handle_occurance(ctx);
            return .continue_with_progress;
        }
        if (self.documentation.names.short) |short| {
            if (std.mem.eql(u8, ctx.args[ctx.cursor], short)) {
                try self.handle_occurance(ctx);
                return .continue_with_progress;
            }
        }
        for (self.documentation.names.aliases) |alias| {
            if (std.mem.eql(u8, ctx.args[ctx.cursor], alias)) {
                try self.handle_occurance(ctx);
                return .continue_with_progress;
            }
        }
        return .continue_no_progress;
    }
    fn handle_occurance(self: *BoolFlag, ctx: *CliContext) !void {
        self.occurances += 1;
        if (self.occurances == 2) {
            try ctx.diagnostics.append(ctx.allocator, .{
                .kind = .warn,
                .message = try std.fmt.allocPrint(ctx.allocator, "Flag '{s}' is present multiple times.", .{self.documentation.names.main}),
            });
        }
        ctx.cursor += 1;
    }
    fn finalize(ptr: *Option, ctx: *CliContext) anyerror!void {
        _ = ptr;
        _ = ctx;
    }
};
