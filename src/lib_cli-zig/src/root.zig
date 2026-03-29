const std = @import("std");
const assert = std.debug.assert;

// TODO: dealloc messages

pub const CliContext = struct {
    args: [][:0]u8,
    allocator: std.mem.Allocator,
    cursor: usize,
    diagnostics: std.ArrayList(Diagnostic),
    documenataion_store: DocumentationStore,

    const DiagnosticKind = enum {
        err,
        help,
        warn,
    };
    const Diagnostic = struct {
        kind: DiagnosticKind,
        message: []const u8,
    };

    const DocumentationStore = struct {
        top_level: Documentation,
        other: std.ArrayList(Documentation),
    };

    pub fn init(allocator: std.mem.Allocator, program_docs: Documentation) !CliContext {
        return .{
            .args = try std.process.argsAlloc(allocator),
            .cursor = 1,
            .diagnostics = std.ArrayList(Diagnostic).empty,
            .documenataion_store = .{
                .top_level = program_docs,
                .other = std.ArrayList(Documentation).empty,
            },
            .allocator = allocator,
        };
    }
    pub fn deinit(self: *CliContext) void {
        std.process.argsFree(self.allocator, self.args);
        assert(self.diagnostics.items.len == 0);
        self.diagnostics.deinit(self.allocator);
    }

    pub fn parse(self: *CliContext, options: []const *Option) !void {
        for (options) |option| {
            try option.add_documentation(option, self);
        }

        var progress_happened = true;
        while (progress_happened) {
            progress_happened = false;
            for (options) |option| {
                if (try option.try_parse_self(option, self)) {
                    progress_happened = true;
                }
            }
        }

        for (options) |option| {
            try option.finalize(option, self);
        }
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

pub const Option = struct {
    add_documentation: *const fn (*Option, *CliContext) std.mem.Allocator.Error!void,
    try_parse_self: *const fn (*Option, *CliContext) anyerror!bool,
    finalize: *const fn (*Option, *CliContext) anyerror!void,
};

pub const BoolFlag = struct {
    present: bool = false,
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

    fn add_documentation(ptr: *Option, ctx: *CliContext) std.mem.Allocator.Error!void {
        const self: *BoolFlag = @fieldParentPtr("vtable", ptr);
        try ctx.documenataion_store.other.append(ctx.allocator, self.documentation);
    }
    fn try_parse(ptr: *Option, ctx: *CliContext) anyerror!bool {
        const self: *BoolFlag = @fieldParentPtr("vtable", ptr);
        // TODO: emit warning if flag is present multiple times
        if (ctx.cursor >= ctx.args.len) return false;
        if (ctx.args[ctx.cursor].len > 2 and
            std.mem.eql(u8, ctx.args[ctx.cursor][0..2], "--") and
            std.mem.eql(u8, ctx.args[ctx.cursor][2..], self.documentation.names.main))
        {
            self.present = true;
            ctx.cursor += 1;
            return true;
        }
        if (self.documentation.names.short) |short| {
            if (ctx.args[ctx.cursor].len > 1 and
                std.mem.eql(u8, ctx.args[ctx.cursor][0..1], "-") and
                std.mem.eql(u8, ctx.args[ctx.cursor][1..], short))
            {
                self.present = true;
                ctx.cursor += 1;
                return true;
            }
        }
        for (self.documentation.names.aliases) |alias| {
            if (ctx.args[ctx.cursor].len > 2 and
                std.mem.eql(u8, ctx.args[ctx.cursor][0..2], "--") and
                std.mem.eql(u8, ctx.args[ctx.cursor][2..], alias))
            {
                self.present = true;
                ctx.cursor += 1;
                return true;
            }
        }
        return false;
    }
    fn finalize(ptr: *Option, ctx: *CliContext) anyerror!void {
        _ = ptr;
        _ = ctx;
    }
};
