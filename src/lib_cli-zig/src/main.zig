const std = @import("std");

const lib_cli_zig = @import("lib_cli_zig");
const CliContext = lib_cli_zig.CliContext;
const CliOption = lib_cli_zig.Option;
const BoolFlag = lib_cli_zig.BoolFlag;

pub fn main() !void {
    const gpa = std.heap.page_allocator;
    var buf: [1024]u8 = undefined;
    var stdout = std.fs.File.stdout().writer(&buf);
    var ctx = try lib_cli_zig.CliContext.init(gpa, .{
        .names = .{ .main = "0installer" },
        .description = "install static executable in path",
    }, &stdout.interface);
    defer ctx.deinit();

    var my_flag = BoolFlag.new(.{
        .names = .{ .main = "--my", .short = "-m", .aliases = &.{"--me"} },
        .description = "my test flag",
    });
    ctx.parse(&.{&my_flag.vtable}) catch |err| switch (err) {
        error.HelpRequested => return,
        else => return err,
    };

    std.debug.print("my_flag.present: {}\n", .{my_flag.present()});
}
