import { vi } from 'vitest'

// Mock Tauri API
vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(),
}))

// Mock Tauri plugin
vi.mock('@tauri-apps/plugin-opener', () => ({
  open: vi.fn(),
}))

// Global test utilities for browser environment
if (typeof globalThis !== 'undefined') {
  globalThis.console = {
    ...console,
    // Uncomment to ignore a specific log level
    // log: vi.fn(),
    // debug: vi.fn(),
    // info: vi.fn(),
    // warn: vi.fn(),
    // error: vi.fn(),
  }
}