const std = @import("std");

const lib_cli_zig = @import("lib_cli_zig");
const CliContext = lib_cli_zig.CliContext;
const CliOption = lib_cli_zig.Option;
const BoolFlag = lib_cli_zig.BoolFlag;

pub fn main() !void {
    const gpa = std.heap.page_allocator;
    var ctx = try lib_cli_zig.CliContext.init(gpa, .{
        .section = null,
        .names = .{ .main = "todo" },
        .description = "asdf asdf",
    });
    defer ctx.deinit();

    var my_flag = BoolFlag.new(.{
        .names = .{ .main = "--my", .short = "-m", .aliases = &.{"--me"} },
        .description = "my test flag",
    });
    try ctx.parse(&.{&my_flag.vtable});

    std.debug.print("my_flag.present: {}\n", .{my_flag.present});
    var buf: [1024]u8 = undefined;
    var stdout = std.fs.File.stdout().writer(&buf);
    try ctx.documenataion_store.print(&stdout.interface);
}
