import { describe, it, expect } from "vitest";
import { getMimeTypeFromPath } from "./chunk";

describe("chunk", () => {
    describe("getMimeTypeFromPath", () => {
        it("should return image/png for .png files", () => {
            expect(getMimeTypeFromPath("image.png")).toBe("image/png");
            expect(getMimeTypeFromPath("path/to/file.PNG")).toBe("image/png");
            expect(getMimeTypeFromPath("C:/Games/screenshot.png")).toBe("image/png");
        });

        it("should return image/jpeg for .jpg and .jpeg files", () => {
            expect(getMimeTypeFromPath("photo.jpg")).toBe("image/jpeg");
            expect(getMimeTypeFromPath("photo.jpeg")).toBe("image/jpeg");
            expect(getMimeTypeFromPath("path/to/file.JPG")).toBe("image/jpeg");
            expect(getMimeTypeFromPath("path/to/file.JPEG")).toBe("image/jpeg");
        });

        it("should return image/gif for .gif files", () => {
            expect(getMimeTypeFromPath("animation.gif")).toBe("image/gif");
            expect(getMimeTypeFromPath("path/to/file.GIF")).toBe("image/gif");
        });

        it("should return image/webp for .webp files", () => {
            expect(getMimeTypeFromPath("modern.webp")).toBe("image/webp");
            expect(getMimeTypeFromPath("path/to/file.WEBP")).toBe("image/webp");
        });

        it("should throw error for unsupported file types", () => {
            expect(() => getMimeTypeFromPath("document.pdf")).toThrow(
                "Unsupported file type"
            );
            expect(() => getMimeTypeFromPath("text.txt")).toThrow(
                "Unsupported file type"
            );
            expect(() => getMimeTypeFromPath("noextension")).toThrow(
                "Unsupported file type"
            );
        });
    });
});
