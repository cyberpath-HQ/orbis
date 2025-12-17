# Orbis Development Guidelines

## Architecture Overview

Orbis is a **Tauri desktop app** with a **plugin-based architecture**:
- **Frontend**: React + TypeScript (Vite), schema-driven UI rendering
- **Backend**: Rust with WASM plugin sandboxing (wasmtime)
- **Two modes**: Standalone (local SQLite) or Client-Server (PostgreSQL)

**Key architectural decisions**:
- Plugins define UI via **JSON schemas** (`PageDefinition`) instead of shipping React code
- State management uses **Zustand + Immer** (page-level stores, not global)
- All plugin-to-UI communication happens through **action executors** (`executeAction`)

## Critical File Structure

```
crates/
├── orbis-plugin/src/ui.rs          # Rust UI schema types (source of truth)
├── orbis-server/src/routes/        # Axum HTTP routes
└── orbis-auth/                     # JWT/Argon2 auth implementation

orbis/
├── src/lib/
│   ├── renderer.tsx                # SchemaRenderer (1600+ lines, renders 30+ component types)
│   ├── actions.ts                  # Action executor system (15+ action types)
│   └── state.ts                    # Zustand store factory + expression evaluator
├── src/types/schema/               # TypeScript schema types (must match Rust)
└── src-tauri/src/
    ├── commands.rs                 # Tauri commands (invoke handlers)
    └── state.rs                    # OrbisState shared across commands
```

## Development Workflows

### Running the App

```bash
cd orbis && bun run tauri dev  # Frontend + backend hot reload
```

**Note**: Frontend runs on Vite dev server, Tauri handles Rust compilation automatically.

### Plugin Development

Plugins are WASM modules with JSON UI schemas:

```bash
cd plugins/hello-plugin
./build.sh  # Builds WASM + adds custom section for manifest
```

**Key pattern**: Plugins don't contain React code. They define `PageDefinition` schemas that the `SchemaRenderer` interprets. See `orbis/src/lib/renderer.tsx` for how schemas → components.

### Testing Auth

No tests exist yet (critical gap). To manually test auth:
1. Run `bun run tauri dev`
2. Login uses Tauri command: `invoke('login', { username, password })`
3. Session stored in `OrbisState.session` (RwLock)
4. Check `orbis/src-tauri/src/commands.rs` for auth implementation

## Code Conventions

### TypeScript/React

**State Management Pattern**:
```tsx
// ✅ Create page-level stores (NOT global)
const stateStore = createPageStateStore({ count: 0 });

// ✅ Use in components
const state = stateStore();
const count = state.count;
state.setState('count', count + 1);
```

**Component Rendering**:
- All UI components in `renderer.tsx` follow `case 'type':` pattern
- Each component checks `schema.visible` expression before rendering
- Use `resolveValue()` to interpolate `${state.field}` expressions

**Error Handling**:
- Wrap plugin pages in `<PluginErrorBoundary pluginId={...}>`
- Wrap sections in `<PageErrorBoundary>`
- Never let errors crash entire app (see `components/error-boundary.tsx`)

### Rust/Tauri

**Adding Tauri Commands**:
1. Add function in `orbis/src-tauri/src/commands.rs` with `#[tauri::command]`
2. Register in `lib.rs`: `tauri::generate_handler![..., my_command]`
3. Call from frontend: `await invoke('my_command', { args })`

**State Access Pattern**:
```rust
#[tauri::command]
async fn my_command(state: State<'_, OrbisState>) -> Result<Value, String> {
    // ✅ Access shared state
    let session = state.get_session();
    // ✅ Use helpers
    if state.is_authenticated() { ... }
}
```

**Plugin System**:
- Plugins loaded from `ORBIS_PLUGINS_DIR` (default: `./plugins`)
- Manifest can be external JSON OR embedded in WASM custom section (`manifest`)
- Plugin routes defined in manifest, handled by `orbis-server/src/routes/plugins.rs`

## Schema System (Critical to Understand)

**Workflow**: Rust types → JSON → Frontend renderer

1. **Rust defines schema** (`crates/orbis-plugin/src/ui.rs`):
   ```rust
   pub enum ComponentSchema {
       Button(ButtonSchema),
       Text(TextSchema),
       // ... 30+ types
   }
   ```

2. **Serialized to JSON** (plugin manifest):
   ```json
   {
     "type": "button",
     "text": "${state.message}",
     "on_click": [{"type": "update_state", "path": "count", "value": "1"}]
   }
   ```

3. **Frontend renders** (`renderer.tsx`):
   ```tsx
   case 'button':
       const text = resolveValue(schema.text, context);
       return <Button onClick={() => executeActions(schema.onClick, context)}>
           {text}
       </Button>;
   ```

**When adding new components**:
1. Add Rust type to `ui.rs`
2. Add TypeScript type to `orbis/src/types/schema/components.ts`
3. Add renderer case to `renderer.tsx`
4. Update action executors if needed (`actions.ts`)

## Common Pitfalls

❌ **Don't create global Zustand stores** - Pages are dynamic plugins, use `createPageStateStore()`

❌ **Don't call React hooks conditionally in renderer** - Each component type is a case branch, hooks go inside

❌ **Don't forget to handle expressions** - `schema.text` might be `"${state.field}"`, always use `resolveValue()`

❌ **Don't modify Rust types without updating TypeScript** - Schema types MUST match or serialization breaks

✅ **Do use error boundaries** - Wrap all plugin content to prevent full crashes

✅ **Do check FEATURE_STATUS.md** - Documents what's production-ready vs. stubbed

## Integration Points

**Tauri ↔ Frontend**:
- Commands invoked via `invoke(command, args)` from `@tauri-apps/api/core`
- State shared through `OrbisState` (Rust) ↔ returned values (TypeScript)

**Plugin ↔ Core**:
- Plugins call core via action: `{"type": "callApi", "api": "core.get_user"}`
- Core calls plugins: Not yet implemented (TODO in action executor)

**Database**:
- SQLx with migrations in `crates/orbis-db/migrations/{postgres,sqlite}/`
- Auto-runs on startup unless `ORBIS_DATABASE_RUN_MIGRATIONS=false`

## Performance Considerations

- `SchemaRenderer` is NOT memoized - consider wrapping in `React.memo` for large schemas
- Expression evaluation (`interpolateExpression`) runs on every render - cache if schemas are static
- WASM plugins are sandboxed (wasmtime) - cross-boundary calls have overhead

## Quick Reference

**Check if feature is ready**: `FEATURE_STATUS.md` (updated Dec 17, 2025)
**Add UI component**: `renderer.tsx` + `types/schema/components.ts` + `ui.rs`
**Add action type**: `actions.ts` + `types/schema/actions.ts` + `ui.rs`
**Debug Tauri commands**: Check `src-tauri/src/commands.rs` + run with `RUST_LOG=debug`
**Plugin manifest schema**: `crates/orbis-plugin/src/manifest.rs`

## Recent Production Changes (Dec 17, 2025)

- ✅ Auth backend now uses real Tauri commands (not stubbed)
- ✅ Error boundaries added at all levels
- ✅ Toast notifications (Sonner) fully integrated
- ✅ Page lifecycle hooks (`onMount`/`onUnmount`) implemented
- ⚠️ TanStack Form integration pending (forms use native HTML currently)

## Final notes

It's essential you keep this file updated with all significant architectural changes, patterns, or conventions as the codebase evolves. This will ensure consistency and maintainability across the team.

Always prioritize security best practices, especially in authentication, data handling, and error management.
Never compromise on code quality, readability, and documentation.
And most importantly struggle to write clean, efficient, and maintainable code that adheres to the established patterns and conventions outlined in this document without leaving any section or code incomplete.

Always output a FULL summary of changes made before ending your response.

If the user asks you to generate a plan file, ALWAYS generate it and as soon as you've solved one or more points immediately update the plan file with the changes made so far.