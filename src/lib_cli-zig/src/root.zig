const std = @import("std");

pub fn bufferedPrint() !void {
    const gpa = std.heap.page_allocator;

    const args = try std.process.argsAlloc(gpa);
    defer std.process.argsFree(gpa, args);
    for (args) |arg| {
        std.debug.print("{s}\n", .{arg});
    }
}
