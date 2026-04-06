/**
 * Stub: plugins/registry.ts
 *
 * The original is 520 lines and pulls in hooks, commands, http-path, and the
 * entire plugin subsystem. The browser chain only needs createEmptyPluginRegistry()
 * and the PluginRegistry type.
 *
 * Upstream: openclaw/src/plugins/registry.ts
 */

export type PluginRegistry = {
  plugins: any[];
  tools: any[];
  hooks: any[];
  typedHooks: any[];
  channels: any[];
  providers: any[];
  gatewayHandlers: Record<string, any>;
  httpHandlers: any[];
  httpRoutes: any[];
  cliRegistrars: any[];
  services: any[];
  commands: any[];
  diagnostics: any[];
};

export function createEmptyPluginRegistry(): PluginRegistry {
  return {
    plugins: [],
    tools: [],
    hooks: [],
    typedHooks: [],
    channels: [],
    providers: [],
    gatewayHandlers: {},
    httpHandlers: [],
    httpRoutes: [],
    cliRegistrars: [],
    services: [],
    commands: [],
    diagnostics: [],
  };
}
