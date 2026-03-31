const std = @import("std");

const lib_cli_zig = @import("lib_cli_zig");
const CliContext = lib_cli_zig.CliContext;
const CliOption = lib_cli_zig.Option;
const BoolFlag = lib_cli_zig.BoolFlag;

pub fn main(init: std.process.Init) !void {
    var buf: [1024]u8 = undefined;
    var stdout = std.Io.File.stdout().writer(init.io, &buf);
    var ctx = try lib_cli_zig.CliContext.init(init.gpa, init.minimal.args, .{
        .names = .{ .main = "0installer" },
        .description = "install static executable in path",
    }, &stdout.interface);
    defer ctx.deinit();

    var my_flag = BoolFlag.new(.{
        .names = .{ .main = "--my", .short = "-m", .aliases = &.{"--me"} },
        .description = "my test flag",
    });
    var subcommand = lib_cli_zig.make_subcommand(.{
        .install = .{
            .name = "install",
            .short_name = "i",
            .aliases = &.{"add"},
            .description = "add executable to your path",
        },
        .update = .{
            .name = "update",
            .description = "updates executable",
        },
    }){};
    ctx.parse(&.{ &my_flag.vtable, &subcommand.vtable }) catch |err| switch (err) {
        error.HelpRequested => return,
        else => return err,
    };
    try ctx.finish();

    std.debug.print("my_flag.present: {}\n", .{my_flag.present()});
    std.debug.print("subcommand.command: {?}\n", .{subcommand.command});
}
