const std = @import("std");

const json_viewer_zig = @import("json_viewer_zig");

pub fn main() !void {
    std.debug.print("\n", .{});

    const gpa = std.heap.page_allocator;
    const args = try std.process.argsAlloc(gpa);
    defer std.process.argsFree(gpa, args);

    if (args.len != 2) {
        std.debug.print("ERROR: you should provide path for json file\n", .{});
        std.process.exit(1);
    }
    const path = args[1];
    const file = try std.fs.cwd().openFile(path, .{});
    var buffer: [16 * 1024]u8 = undefined;
    var reader = file.reader(&buffer);
    const size = try reader.getSize();

    // Prints to stderr, ignoring potential errors.
    std.debug.print("size: '{any}'\n", .{size});

    try json_viewer_zig.bufferedPrint();
}
