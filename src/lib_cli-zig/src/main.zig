const std = @import("std");

const lib_cli_zig = @import("lib_cli_zig");

pub fn main() !void {
    try lib_cli_zig.bufferedPrint();
}
