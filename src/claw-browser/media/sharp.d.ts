/**
 * Stub type declaration for sharp (optional dependency).
 * sharp is lazily imported at runtime only when image resizing is needed.
 * If sharp is not installed, the browser subsystem falls back gracefully.
 */
declare module "sharp" {
  interface SharpInstance {
    metadata(): Promise<{ width?: number; height?: number; format?: string }>;
    resize(width?: number, height?: number, options?: any): SharpInstance;
    jpeg(options?: { quality?: number }): SharpInstance;
    png(options?: any): SharpInstance;
    toBuffer(): Promise<Buffer>;
  }

  function sharp(input?: Buffer | string, options?: any): SharpInstance;

  export default sharp;
}
