# Orbis DSL Improvements Roadmap

> **Created**: December 24, 2025  
> **Last Updated**: December 26, 2025  
> **Branch**: `feature/pest-based-dsl-for-page-definitions-plugin-metadata-dsl`

This document outlines the current state of the Orbis DSL and proposes improvements to make it more powerful, clean, easy to read and migrate to, while providing excellent developer experience (DX).

## Current State Analysis

### ✅ What We Have Now (December 2025)

#### Core Language Features
- **JSX-like component syntax** with self-closing and paired tags
- **Expression system** supporting:
  - Member access (`state.field`, `item.property`)
  - Binary operators (`==`, `!=`, `&&`, `||`, `+`, `-`, `*`, `/`, `%`)
  - Unary operators (`!`, `-`)
  - String interpolation (`"Hello, {state.name}!"`)
- **State declarations** with optional type annotations
- **Lifecycle hooks** in dedicated `hooks` block (`@mount`, `@unmount`)
- **Control flow structures**:
  - `if` statements with optional `else`
  - `for` loops with iterators
  - `when` pattern matching
- **Action system** with response handlers (`success`, `error`, `finally`)
- **Component whitelisting** (23 built-in components)
- **Attribute whitelisting** per component with descriptions
- **Event whitelisting** per component
- **Strongly typed attribute values** (AlertType, ButtonVariant, etc.)
- **Deprecation support** for components, attributes, and events
- **Auto-generated documentation** (COMPONENT_REFERENCE.md)

#### Build System
- **Pest-based parser** (v2.8.4) with grammar generation
- **Build script automation** for keyword and component rules
- **HashSet-based deduplication** preventing duplicate rules
- **Multi-grammar support** (page.pest, metadata.pest)

#### Testing
- **21 passing tests** covering parsing and validation
- **Example validation** ensuring grammar correctness
- **Attribute/event whitelisting tests**
- **51+ passing tests** covering all new features (fragments, watchers, enhanced types, imports, validation, CSS)

### 📊 Current Limitations

1. **No IDE support** - No LSP, syntax highlighting, or autocomplete
2. **Basic error messages** - Pest errors are technical, not user-friendly
3. ~~**No modularity** - Can't import/reuse page fragments or components~~ → ✅ **RESOLVED** - Fragments + Import/Export
4. ~~**No custom components** - Limited to 23 built-in components~~ → Now 32 components
5. ~~**No CSS integration** - Styling requires className strings~~ → ✅ **RESOLVED** - CSS-in-DSL
6. **No i18n support** - Hardcoded strings only
7. ~~**No computed properties** - State is static declarations only~~ → ✅ **RESOLVED**
8. ~~**No validation rules** - Beyond schema, no custom validation~~ → ✅ **RESOLVED** - Zod v4 validation
9. **No debugging tools** - No dev mode, error boundaries, or tracing
10. **No formatter** - Manual code formatting required
11. ~~**Basic type system** - No unions, generics, interfaces~~ → ✅ **RESOLVED** - Enhanced types
12. ~~**No watchers** - Can't react to state changes~~ → ✅ **RESOLVED** - Watcher hooks

---

## Priority 1: Essential DX Improvements (Must-Have)

### 1.1 Language Server Protocol (LSP) Implementation
**Impact**: 🔥 Critical | **Effort**: 🏗️ High | **Timeline**: 2-3 weeks

**Why**: IDE integration is the #1 request from developers. Without autocomplete, diagnostics, and jump-to-definition, adoption will be slow.

**What to implement**:
- **Autocomplete** for:
  - Component names (all whitelisted)
  - Attribute names (context-aware per component)
  - Event names (whitelisted per component)
  - State variable names (from state block)
  - Action types (from action system)
- **Hover documentation** showing:
  - Component descriptions
  - Attribute descriptions and allowed values
  - Deprecation warnings with alternatives
- **Diagnostics** with:
  - Friendly error messages (not Pest internals)
  - Quick fixes for common mistakes
  - "Did you mean X?" suggestions
- **Go-to-definition** for state variables
- **Rename refactoring** for state variables
- **Document symbols** for outline view

**Technical approach**:
- Create `crates/orbis-lsp` using `tower-lsp`
- Reuse existing Pest parser for syntax analysis
- Build symbol table from AST
- Integrate with component definitions from build.rs

**VS Code extension structure**:
```
orbis-vscode/
├── package.json
├── syntaxes/
│   └── orbis.tmLanguage.json  # Syntax highlighting
├── language-configuration.json
└── src/
    └── extension.ts           # LSP client
```

**Example user experience**:
```orbis
template {
    <Bu|  ← Autocomplete shows: Button, Badge
    <Button la|  ← Shows: label, loading (with descriptions)
}
```

---

### 1.2 Rich Error Messages with Recovery Suggestions
**Impact**: 🔥 Critical | **Effort**: 🛠️ Medium | **Timeline**: 1 week

**Why**: Current Pest errors like `positives: [FieldAttributes]` are cryptic. Developers need actionable guidance.

**What to implement**:
- **Custom error types** replacing Pest errors:
  ```rust
  pub enum OrbisError {
      UnknownComponent { name: String, line: usize, suggestions: Vec<String> },
      InvalidAttribute { component: String, attr: String, valid: Vec<String> },
      MissingRequiredAttribute { component: String, attr: String },
      DeprecatedUsage { item: String, replacement: String, docs_url: String },
      ExpressionError { expr: String, reason: String },
      // ...
  }
  ```
- **Fuzzy matching** for suggestions:
  ```
  Error: Unknown component 'Buttom'
  ├─ at line 12, column 5
  ├─ Did you mean 'Button'?
  └─ See available components: https://docs.orbis.dev/components
  ```
- **Multi-error reporting** (don't stop at first error)
- **Color-coded output** with `annotate-snippets` crate
- **Fix suggestions** in JSON format for LSP:
  ```json
  {
    "diagnostics": [{
      "message": "Unknown attribute 'name'",
      "quickFixes": [
        { "title": "Use 'fieldName'", "edit": "..." }
      ]
    }]
  }
  ```

**Before**:
```
Error: positives: [FieldAttributes, FieldEventsDefinition]
at line 44, column 17
```

**After**:
```
Error: Invalid attribute 'name' on <Field> component
  ┌─ example.orbis:44:17
  │
44│             <Field name="username" />
  │                    ^^^^ unknown attribute
  │
  = help: Did you mean 'fieldName'?
  = note: Valid attributes: id, className, type, fieldName, label, placeholder
  = docs: https://docs.orbis.dev/components/Field
```

---

### 1.3 Formatter (orbis fmt)
**Impact**: 🔥 Critical | **Effort**: 🛠️ Medium | **Timeline**: 1 week

**Why**: Consistent formatting reduces bike-shedding and improves readability. Should be like `rustfmt` or `prettier`.

**What to implement**:
- **CLI tool**: `orbis fmt [files]`
- **Auto-formatting rules**:
  - Indent: 4 spaces (configurable)
  - Max line length: 100 chars (configurable)
  - Component attribute wrapping at 3+ attributes
  - Consistent spacing around operators
  - Sorted state declarations (optional)
  - Sorted attributes alphabetically (optional)
- **Configuration file**: `orbis.toml`
  ```toml
  [format]
  indent_size = 4
  max_line_width = 100
  sort_attributes = false
  trailing_commas = true
  ```
- **VS Code integration**: Format on save

**Before**:
```orbis
<Button label="Click" type="button" variant="primary" disabled={state.loading} @click=>[state.count=state.count+1]/>
```

**After**:
```orbis
<Button 
    label="Click"
    type="button" 
    variant="primary"
    disabled={state.loading}
    @click => [
        state.count = state.count + 1
    ]
/>
```

---

### 1.4 Component Fragments and Composition ✅ COMPLETED
**Impact**: 🔥 High | **Effort**: 🛠️ Medium | **Timeline**: 1 week

> **Status**: ✅ Fully implemented with typed parameters, event passing, and named/unnamed slots.
> See "Recently Completed (December 26, 2025)" section for details.

**What was implemented**:
- **Fragment definitions** in same file:
  ```orbis
  fragment UserCard(user: User) {
      <Card className="user-card">
          <Heading level="2" content={user.name} />
          <Text content={user.email} />
          <Badge content={user.role} variant="info" />
      </Card>
  }
  
  template {
      for user in state.users {
          <UserCard user={user} />
      }
  }
  ```
- **Fragment parameters** with type annotations
- **Slot system** for content projection:
  ```orbis
  fragment Modal(title: string, open: boolean) {
      <Modal title={title} open={open}>
          <slot />  <!-- Content goes here -->
      </Modal>
  }
  
  template {
      <Modal title="Confirm" open={state.showModal}>
          <Text content="Are you sure?" />
          <Button label="Confirm" />
      </Modal>
  }
  ```
- **Named slots**:
  ```orbis
  fragment Layout {
      <Container>
          <slot name="header" />
          <slot name="content" />
          <slot name="footer" />
      </Container>
  }
  ```

**Validation**:
- Fragment parameters must be used
- Slot content must match fragment expectations
- Recursive fragments prevented

---

### 1.5 Import System for Modularity ✅ COMPLETED
**Impact**: 🔥 High | **Effort**: 🏗️ High | **Timeline**: 2 weeks

> **Status**: ✅ Fully implemented with TypeScript and Rust-style syntax.
> See "Recently Completed (December 26, 2025) - Part 2" section for complete details.

**What was implemented**:
- **Import syntax**:
  ```orbis
  import { UserCard, PostCard } from "./fragments/cards.orbis"
  import * as Utils from "./utils.orbis"
  import { API_BASE_URL } from "./constants.orbis"
  ```
- **Export syntax**:
  ```orbis
  // fragments/cards.orbis
  export fragment UserCard(user: User) { ... }
  export fragment PostCard(post: Post) { ... }
  export const API_BASE_URL = "https://api.example.com"
  ```
- **Relative imports** (`./, ../`)
- **Package imports** (`@orbis/common`, `@myorg/ui`)
- **Circular dependency detection**
- **Tree shaking** for unused exports

**Project structure**:
```
src/
├── pages/
│   ├── home.orbis
│   └── profile.orbis
├── fragments/
│   ├── cards.orbis
│   └── forms.orbis
└── constants.orbis
```

**Build-time resolution**:
- Resolve imports during build
- Generate dependency graph
- Bundle for runtime (single-file output or module system)

---

## Priority 2: Enhanced Language Features (Should-Have)

### 2.1 Computed Properties and Getters ✅ COMPLETED
**Impact**: 🔥 High | **Effort**: 🛠️ Medium | **Timeline**: 4 days

**Why**: Derived state is common (e.g., `fullName = firstName + lastName`). Avoids manual synchronization.

**What to implement**:
```orbis
state {
    firstName: string = "John"
    lastName: string = "Doe"
    count = 0
    
    // Computed property (read-only)
    @computed fullName: string => {firstName} + " " + {lastName}
    
    // Computed with multiple dependencies
    @computed isEven: boolean => {count} % 2 == 0
    
    // Computed with complex logic
    @computed greeting: string => {
        if {count} > 10 {
            "Wow, " + {fullName} + "!"
        } else {
            "Hello, " + {fullName}
        }
    }
}

template {
    <Text content={state.fullName} />  <!-- Auto-updates -->
}
```

**Implementation**:
- Mark computed properties with `@computed` decorator
- Track dependencies from expression analysis
- Auto-recompute when dependencies change (frontend responsibility)
- Validation: computed properties can't be assigned to

---

### 2.2 Watchers for Side Effects ✅ COMPLETED
**Impact**: 🟡 Medium | **Effort**: 🛠️ Medium | **Timeline**: 3 days

> **Status**: ✅ Fully implemented with debounce, immediate, deep options.
> See "Recently Completed (December 26, 2025)" section for details.

**What was implemented**:
```orbis
state {
    count = 0
    searchQuery = ""
}

hooks {
    @mount => [
        console.log("Mounted")
    ]
    
    @watch(state.count) => [
        console.log("Count changed to: {state.count}"),
        localStorage.set("count", {state.count})
    ]
    
    @watch(state.searchQuery, debounce: 300) => [
        api.call("search", query: {state.searchQuery})
    ]
}
```

**Features**:
- Watch single state property
- Watch multiple properties: `@watch(state.a, state.b)`
- Debounce option for performance
- Access to old value: `@watch(state.count) => [console.log({$oldValue}, {$newValue})]`

---

### 2.3 Validation Rules and Constraints ✅ COMPLETED
**Impact**: 🟡 Medium | **Effort**: 🛠️ Medium | **Timeline**: 5 days

> **Status**: ✅ Fully implemented with Zod v4-compatible syntax and all validators.
> See "Recently Completed (December 26, 2025) - Part 2" section for complete details.

**Why**: Form validation is common. Should be declarative, not imperative.

**What was implemented**:
```orbis
state {
    email: string = "" {
        @validate required: true
        @validate pattern: "^[^@]+@[^@]+\\.[^@]+$"
        @validate message: "Please enter a valid email"
    }
    
    age: number = 0 {
        @validate min: 18
        @validate max: 120
        @validate message: "Age must be between 18 and 120"
    }
    
    password: string = "" {
        @validate minLength: 8
        @validate custom: hasUppercase && hasNumber
    }
}

template {
    <Field 
        fieldName="email" 
        bind={state.email}
        error={state.$errors.email}  <!-- Auto-populated -->
    />
}
```

**Built-in validators**:
- `required`, `minLength`, `maxLength`, `min`, `max`
- `pattern` (regex), `email`, `url`, `numeric`
- `custom` (expression-based)

**Auto-generated state properties**:
- `state.$errors.fieldName` - error message or null
- `state.$valid` - boolean, true if all fields valid
- `state.$touched.fieldName` - boolean, true if field interacted with

---

### 2.4 Type System Enhancements ✅ COMPLETED
**Impact**: 🟡 Medium | **Effort**: 🏗️ High | **Timeline**: 2 weeks

> **Status**: ✅ Fully implemented with unions, optionals, interfaces, generics, and special types.
> See "Recently Completed (December 26, 2025)" section for details.

**What was implemented**:
- **Union types**:
  ```orbis
  state {
      status: "idle" | "loading" | "success" | "error" = "idle"
      data: User | null = null
  }
  ```
- **Optional types**:
  ```orbis
  state {
      user?: User  // Equivalent to User | null
      config?: Config = defaultConfig
  }
  ```
- **Interface definitions**:
  ```orbis
  interface User {
      id: number
      name: string
      email: string
      role: "admin" | "user" | "guest"
  }
  
  state {
      currentUser: User = { id: 1, name: "John", email: "john@example.com", role: "user" }
  }
  ```
- **Type inference** from initial values:
  ```orbis
  state {
      count = 0           // Infers number
      name = "John"       // Infers string
      items = []          // Infers any[]
      items: Item[] = []  // Explicit better
  }
  ```
- **Generic types**:
  ```orbis
  interface Response<T> {
      data: T
      error: string | null
  }
  
  state {
      userResponse: Response<User> = { data: null, error: null }
  }
  ```

---

### 2.5 CSS-in-DSL Support ✅ COMPLETED
**Impact**: 🟡 Medium | **Effort**: 🛠️ Medium | **Timeline**: 1 week

> **Status**: ✅ Fully implemented with scoped/global styles, Tailwind integration, and all CSS at-rules.
> See "Recently Completed (December 26, 2025) - Part 2" section for complete details.

**Why**: Inline styles are verbose. CSS classes require external files. Need middle ground.

**What was implemented**:
- **Style block** with scoped CSS:
  ```orbis
  styles {
      .card {
          padding: 1rem;
          border-radius: 8px;
          box-shadow: 0 2px 4px rgba(0,0,0,0.1);
      }
      
      .card.highlighted {
          border: 2px solid blue;
      }
      
      @media (max-width: 768px) {
          .card {
              padding: 0.5rem;
          }
      }
  }
  
  template {
      <Container className="card" />
  }
  ```
- **Style interpolation**:
  ```orbis
  styles {
      .dynamic {
          color: {state.theme.primaryColor};
          font-size: {state.fontSize}px;
      }
  }
  ```
- **Auto-scoping** to prevent leaks (like CSS Modules)
- **Tailwind integration** (parse utilities, generate CSS)

---

### 2.6 Internationalization (i18n) System
**Impact**: 🟡 Medium | **Effort**: 🏗️ High | **Timeline**: 2 weeks

**Why**: Multi-language support is required for global apps. Should be first-class.

**What to implement**:
- **Translation files**:
  ```json
  // locales/en.json
  {
    "greeting": "Hello, {name}!",
    "buttons": {
      "submit": "Submit",
      "cancel": "Cancel"
    },
    "validation": {
      "required": "This field is required"
    }
  }
  ```
- **i18n syntax**:
  ```orbis
  import { t } from "@orbis/i18n"
  
  state {
      locale: "en" | "es" | "fr" = "en"
      userName = "John"
  }
  
  template {
      <Text content={t("greeting", name: state.userName)} />
      <Button label={t("buttons.submit")} />
  }
  ```
- **Pluralization**:
  ```orbis
  <Text content={t("items_count", count: state.items.length)} />
  // "1 item" or "5 items"
  ```
- **Date/number formatting**:
  ```orbis
  <Text content={formatDate(state.createdAt, locale: state.locale)} />
  <Text content={formatNumber(state.price, locale: state.locale, style: "currency")} />
  ```
- **Hot-reload** translations in dev mode

---

## Priority 3: Developer Tools (Nice-to-Have)

### 3.1 CLI Tool Suite
**Impact**: 🟡 Medium | **Effort**: 🛠️ Medium | **Timeline**: 1 week

**What to implement**:
```bash
# Create new page from template
orbis new page profile --template=dashboard

# Validate all .orbis files
orbis check src/

# Format files
orbis fmt src/

# Build and bundle
orbis build --output=dist/

# Dev server with hot reload
orbis dev --port=3000

# Run linter
orbis lint src/

# Generate TypeScript types from state
orbis types src/ --output=types/

# Migrate from other frameworks
orbis migrate --from=react components/App.jsx
```

**Templates**:
```
orbis new page <name>
  --template=blank        # Minimal page
  --template=form         # Form with fields
  --template=dashboard    # Grid layout with cards
  --template=crud         # List + detail + edit
```

---

### 3.2 Testing Utilities
**Impact**: 🟡 Medium | **Effort**: 🏗️ High | **Timeline**: 2 weeks

**Why**: Untested DSL pages are brittle. Need first-class testing support.

**What to implement**:
- **Test block syntax**:
  ```orbis
  test "counter increments on click" {
      // Arrange
      mount()
      expect(state.count).toBe(0)
      
      // Act
      click(<Button label="Increment" />)
      
      // Assert
      expect(state.count).toBe(1)
  }
  
  test "form validation works" {
      mount()
      
      // Empty field should show error
      submit(<Form />)
      expect(state.$errors.email).toBe("Email is required")
      
      // Valid email should clear error
      type(<Field fieldName="email" />, "test@example.com")
      expect(state.$errors.email).toBe(null)
  }
  ```
- **Test runners** integration (Jest, Vitest)
- **Snapshot testing** for rendered output
- **Mock actions**: `mockAction("api.call", returns: { data: mockUser })`

---

### 3.3 Visual Page Builder (Optional)
**Impact**: 🟢 Low | **Effort**: 🏗️ Very High | **Timeline**: 4+ weeks

**Why**: Not all developers want to write code. Drag-and-drop can speed up prototyping.

**What to implement**:
- Web-based editor with:
  - Component palette (drag components to canvas)
  - Property inspector (edit attributes)
  - Visual state editor
  - Live preview
  - Two-way sync with .orbis files (edit code or visual)
- **Generate clean DSL code** (no bloat)
- **Not a replacement** for code editing (complement)

---

## Priority 4: Advanced Features (Future)

### 4.1 Plugin System for Custom Actions
**Impact**: 🟢 Low | **Effort**: 🏗️ High | **Timeline**: 3 weeks

**Why**: Not all actions can be built-in. Developers need extensibility.

**What to implement**:
```orbis
// Plugin definition
plugin Analytics {
    action track(event: string, properties: object) {
        // Rust/WASM implementation
    }
}

// Usage
template {
    <Button 
        @click => [
            analytics.track("button_clicked", { label: "Submit" })
        ]
    />
}
```

---

### 4.2 Async/Await Syntax for Actions
**Impact**: 🟢 Low | **Effort**: 🛠️ Medium | **Timeline**: 5 days

**Why**: Current `api.call(...) { success => [...] }` is verbose for simple cases.

**What to implement**:
```orbis
template {
    <Button @click => [
        state.loading = true,
        let response = await api.call("users"),
        state.users = response.data,
        state.loading = false
    ] />
}
```

**Error handling**:
```orbis
try [
    let user = await api.call("user", id: state.userId),
    state.currentUser = user
] catch error [
    toast.show("Failed: " + error.message, level: error)
]
```

---

### 4.3 Hot Module Replacement (HMR)
**Impact**: 🟢 Low | **Effort**: 🏗️ High | **Timeline**: 2 weeks

**Why**: Full page reloads lose state. HMR preserves state during development.

**What to implement**:
- Detect file changes
- Re-parse DSL
- Diff old vs new page structure
- Apply minimal changes to DOM
- Preserve state unless state block changed

---

### 4.4 Performance Optimization Hints
**Impact**: 🟢 Low | **Effort**: 🛠️ Medium | **Timeline**: 1 week

**What to implement**:
```orbis
// Lazy load component
<ExpensiveChart @lazy />

// Memoize expensive computation
@computed @memo filteredItems: Item[] => {
    state.items.filter(item => item.active)
}

// Virtual scrolling for long lists
<List 
    items={state.items}
    virtual={true}
    itemHeight={50}
/>
```

---

### 4.5 Accessibility Linter
**Impact**: 🟢 Low | **Effort**: 🛠️ Medium | **Timeline**: 1 week

**Why**: Accessibility is often an afterthought. Built-in linting can catch issues early.

**Rules**:
- Images must have `alt` text
- Buttons must have `label` or `aria-label`
- Form fields must have `label`
- Heading levels must be sequential (no h1 → h3)
- Color contrast warnings
- Keyboard navigation checks

```
Warning: <Button> missing 'label' attribute
  ┌─ page.orbis:15:5
  │
15│     <Button @click => {...} />
  │     ^^^^^^ add label for screen readers
  │
  = help: Add label="Click here" or aria-label="Close dialog"
```

---

## Priority 5: Innovative Power Features (Vision)

These features represent the next evolution of the Orbis DSL, pushing beyond traditional DSL capabilities to enable truly powerful and ergonomic development experiences.

### 5.1 Reactive Expressions with Fine-Grained Reactivity
**Impact**: 🔥 High | **Effort**: 🏗️ High | **Timeline**: 3 weeks

**Why**: Current reactivity requires explicit watchers. Fine-grained reactivity (like SolidJS) enables automatic dependency tracking.

**What to implement**:

```orbis
state {
    items: Item[] = []
    filter: string = ""
    
    // Auto-reactive: tracks dependencies automatically
    @reactive filteredItems => items.filter(item => item.name.includes(filter))
    
    // Memo with manual invalidation
    @memo(deps: [items, filter]) expensiveComputation => {
        // Only recalculates when deps change
        items.reduce((acc, item) => /* expensive */, 0)
    }
}

template {
    // Only re-renders when filteredItems actually changes
    <List items={state.filteredItems} />
}
```

**Features**:
- Automatic dependency tracking
- Memoization with explicit or inferred dependencies
- Batched updates for performance
- Debug mode showing dependency graph

---

### 5.2 Effect System for Side Effects
**Impact**: 🔥 High | **Effort**: 🛠️ Medium | **Timeline**: 1 week

**Why**: Side effects (logging, analytics, persistence) should be declarative and composable.

**What to implement**:

```orbis
effects {
    // Auto-persist state changes to localStorage
    @persist(state.settings, key: "user-settings")
    
    // Sync state with URL query params
    @urlSync(state.page, param: "p")
    @urlSync(state.filter, param: "q")
    
    // Broadcast state changes to other tabs
    @broadcast(state.theme, channel: "theme-sync")
    
    // Debounced analytics
    @track(state.searchQuery, event: "search", debounce: 500)
    
    // Undo/redo history
    @history(state.document, maxSize: 50)
}

template {
    <Button @click => { state.document.undo() } label="Undo" />
    <Button @click => { state.document.redo() } label="Redo" />
}
```

**Built-in Effects**:
- `@persist` - localStorage/sessionStorage sync
- `@urlSync` - URL query parameter binding
- `@broadcast` - Cross-tab communication
- `@track` - Analytics event tracking
- `@history` - Undo/redo stack
- `@log` - Console logging for debugging

---

### 5.3 Declarative Animations and Transitions
**Impact**: 🟡 Medium | **Effort**: 🛠️ Medium | **Timeline**: 1 week

**Why**: Animations are complex in code. DSL can provide high-level animation primitives.

**What to implement**:

```orbis
animations {
    fadeIn {
        from { opacity: 0 }
        to { opacity: 1 }
        duration: 300ms
        easing: ease-out
    }
    
    slideUp {
        from { transform: translateY(20px), opacity: 0 }
        to { transform: translateY(0), opacity: 1 }
        duration: 400ms
        easing: cubic-bezier(0.4, 0, 0.2, 1)
    }
    
    stagger {
        animation: fadeIn
        delay: 50ms  // Delay between each item
    }
}

template {
    // Entrance animation
    <Card @enter={fadeIn}>
        <Text content="Hello" />
    </Card>
    
    // Exit animation
    if state.visible {
        <Modal @enter={slideUp} @exit={slideUp.reverse}>
            <Text content="Modal content" />
        </Modal>
    }
    
    // Staggered list animation
    <List items={state.items} @enter={stagger}>
        <Card>{item.name}</Card>
    </List>
    
    // Layout animations (FLIP)
    <Container @layout>
        for item in state.sortedItems {
            <Card key={item.id}>{item.name}</Card>
        }
    </Container>
}
```

**Features**:
- Declarative keyframe definitions
- Enter/exit transitions
- Staggered animations for lists
- Layout animations (FLIP technique)
- Spring physics option
- Prebuilt animation library

---

### 5.4 State Machines and Statecharts
**Impact**: 🟡 Medium | **Effort**: 🛠️ Medium | **Timeline**: 1 week

**Why**: Complex UI state (wizards, modals, async flows) is error-prone. State machines formalize transitions.

**What to implement**:

```orbis
machine AuthFlow {
    initial: idle
    
    states {
        idle {
            on LOGIN_CLICK => loading
        }
        
        loading {
            entry => { api.login(state.credentials) }
            on SUCCESS => authenticated
            on ERROR => error
            on TIMEOUT(5000) => error
        }
        
        authenticated {
            entry => { state.user = $event.user }
            on LOGOUT => idle
        }
        
        error {
            entry => { state.errorMessage = $event.message }
            on RETRY => loading
            on DISMISS => idle
        }
    }
    
    guards {
        hasCredentials => state.email && state.password
    }
}

template {
    when AuthFlow.state {
        is idle {
            <LoginForm @submit => { AuthFlow.send(LOGIN_CLICK) } />
        }
        is loading {
            <Spinner />
        }
        is authenticated {
            <Dashboard user={state.user} />
        }
        is error {
            <Alert type="error" message={state.errorMessage} />
            <Button @click => { AuthFlow.send(RETRY) } label="Retry" />
        }
    }
}
```

**Features**:
- XState-compatible syntax
- Entry/exit actions
- Guards for conditional transitions
- Hierarchical (nested) states
- Parallel states
- History states
- Timeout transitions
- Debug visualization

---

### 5.5 Query Builder for Data Fetching
**Impact**: 🟡 Medium | **Effort**: 🛠️ Medium | **Timeline**: 1 week

**Why**: Data fetching is repetitive. A declarative query system (like TanStack Query) reduces boilerplate.

**What to implement**:

```orbis
queries {
    users: User[] {
        fetch => api.get("/users")
        staleTime: 5m
        cacheTime: 30m
        refetchOnFocus: true
    }
    
    user(id: string): User {
        fetch => api.get("/users/{id}")
        enabled => id != null
        retry: 3
    }
    
    createUser: Mutation<CreateUserInput, User> {
        mutate => api.post("/users", $input)
        onSuccess => { 
            queries.users.invalidate()
            toast.show("User created")
        }
        onError => { toast.show($error.message, level: error) }
    }
}

template {
    when queries.users {
        is loading { <Spinner /> }
        is error { <Alert message={queries.users.error} /> }
        is success {
            for user in queries.users.data {
                <UserCard user={user} />
            }
        }
    }
    
    <Button 
        @click => { queries.createUser.mutate({ name: state.name }) }
        loading={queries.createUser.isLoading}
        label="Create User"
    />
}
```

**Features**:
- Automatic caching and invalidation
- Background refetching
- Optimistic updates
- Infinite queries (pagination)
- Dependent queries
- Prefetching
- Query devtools integration

---

### 5.6 Server Components and SSR Support
**Impact**: 🔥 High | **Effort**: 🏗️ Very High | **Timeline**: 4+ weeks

**Why**: Server rendering improves performance and SEO. Orbis should support hybrid rendering.

**What to implement**:

```orbis
// Server-only component (no JS shipped to client)
@server
fragment StaticHeader {
    <Header>
        <Text content={await db.getSiteTitle()} />
        <Navigation items={await db.getNavItems()} />
    </Header>
}

// Client component (interactive)
@client
fragment Counter {
    state { count: number = 0 }
    
    template {
        <Button @click => { state.count++ } label={state.count} />
    }
}

// Hybrid: server-rendered with client hydration
fragment ProductPage(id: string) {
    // This runs on server
    let product = await db.getProduct(id)
    
    template {
        <StaticHeader />  // Server component
        <ProductDetails product={product} />
        <Counter />  // Client component (island)
    }
}
```

**Features**:
- `@server` decorator for server-only components
- `@client` decorator for client-only components
- Automatic code splitting
- Streaming SSR support
- Islands architecture
- Partial hydration

---

### 5.7 AI-Assisted Development Integration
**Impact**: 🔥 High | **Effort**: 🏗️ High | **Timeline**: 2 weeks

**Why**: AI can accelerate development. First-class AI integration makes Orbis AI-native.

**What to implement**:

```orbis
// AI-generated component from natural language
@ai "A card showing user avatar, name, email, and a follow button"
fragment UserCard

// AI-assisted form generation
@ai "A contact form with name, email, subject, and message fields with validation"
fragment ContactForm

// AI content in templates
template {
    <Text content={@ai.complete("Friendly greeting for {state.userName}")} />
    
    // AI-powered search
    <SearchResults 
        items={@ai.semanticSearch(state.items, query: state.query)} 
    />
    
    // AI classification
    <Badge 
        variant={@ai.classify(state.text, categories: ["positive", "negative", "neutral"])}
    />
}

// AI-powered validation
state {
    bio: string = "" | @ai.moderate(policy: "no-profanity") | maxLength(500)
}
```

**Features**:
- Component generation from descriptions
- Semantic search in data
- Content classification
- Text moderation
- Translation assistance
- Code suggestions in LSP

---

### 5.8 Type-Safe Action Pipelines
**Impact**: 🟡 Medium | **Effort**: 🛠️ Medium | **Timeline**: 1 week

**Why**: Complex action chains are hard to read. Pipelines make data flow explicit.

**What to implement**:

```orbis
pipelines {
    submitForm {
        validate(state.form)
        |> transform(data => ({ ...data, timestamp: now() }))
        |> api.post("/submit")
        |> onSuccess(response => {
            state.result = response.data
            toast.show("Submitted!")
        })
        |> onError(error => {
            state.errors = error.details
            toast.show(error.message, level: error)
        })
        |> finally(() => { state.loading = false })
    }
    
    fetchAndProcess {
        api.get("/data")
        |> map(response => response.items)
        |> filter(item => item.active)
        |> sort((a, b) => a.name.localeCompare(b.name))
        |> tap(items => { state.items = items })
    }
}

template {
    <Button 
        @click => { pipelines.submitForm.run() }
        label="Submit"
    />
}
```

**Features**:
- Pipe operator (`|>`) for chaining
- Built-in operators: `map`, `filter`, `sort`, `tap`, `reduce`
- Error handling with `onError`
- Cancellation support
- Progress tracking
- Retry policies

---

### 5.9 Design Tokens and Theming System
**Impact**: 🟡 Medium | **Effort**: 🛠️ Medium | **Timeline**: 1 week

**Why**: Design consistency requires tokens. DSL should natively support design systems.

**What to implement**:

```orbis
tokens {
    colors {
        primary: #3b82f6
        primary.hover: #2563eb
        primary.active: #1d4ed8
        
        semantic {
            success: #22c55e
            warning: #f59e0b
            error: #ef4444
            info: #3b82f6
        }
        
        // Auto-generate dark mode variants
        @dark {
            primary: #60a5fa
            background: #1f2937
        }
    }
    
    spacing {
        xs: 0.25rem
        sm: 0.5rem
        md: 1rem
        lg: 1.5rem
        xl: 2rem
    }
    
    typography {
        font.sans: "Inter, system-ui, sans-serif"
        font.mono: "Fira Code, monospace"
        
        size {
            sm: 0.875rem
            base: 1rem
            lg: 1.125rem
            xl: 1.25rem
        }
    }
    
    radii {
        sm: 0.25rem
        md: 0.5rem
        lg: 1rem
        full: 9999px
    }
}

styles {
    .button {
        padding: $spacing.sm $spacing.md;
        border-radius: $radii.md;
        font-family: $typography.font.sans;
        background: $colors.primary;
        
        &:hover {
            background: $colors.primary.hover;
        }
    }
}

template {
    // Token access in attributes
    <Container padding={$spacing.lg} background={$colors.semantic.info} />
}
```

**Features**:
- Token definitions with dot notation
- Automatic dark mode generation
- Token validation and type checking
- Export to CSS variables, Tailwind config, Figma
- Design system documentation generation

---

### 5.10 Macro System for Code Generation
**Impact**: 🟡 Medium | **Effort**: 🏗️ High | **Timeline**: 2 weeks

**Why**: Repetitive patterns benefit from macros. Reduces boilerplate while maintaining clarity.

**What to implement**:

```orbis
// Define a macro
macro CRUD_Page(entity: string, fields: Field[]) {
    state {
        items: ${entity}[] = []
        selected: ${entity}? = null
        isEditing: boolean = false
    }
    
    hooks {
        @mount => { api.load("${entity.toLowerCase()}s") }
    }
    
    template {
        <DataTable 
            columns={[
                ${for field in fields}
                { key: "${field.name}", label: "${field.label}" },
                ${endfor}
            ]}
            data={state.items}
            @rowClick => { state.selected = $event.row }
        />
        
        if state.selected {
            <Modal title="Edit ${entity}">
                ${for field in fields}
                <Field 
                    fieldName="${field.name}"
                    label="${field.label}"
                    type="${field.type}"
                    bind={state.selected.${field.name}}
                />
                ${endfor}
                <Button @click => { api.update("${entity.toLowerCase()}s", state.selected) } label="Save" />
            </Modal>
        }
    }
}

// Use the macro
@CRUD_Page(
    entity: "User",
    fields: [
        { name: "name", label: "Name", type: "text" },
        { name: "email", label: "Email", type: "email" },
        { name: "role", label: "Role", type: "select" }
    ]
)
```

**Features**:
- Compile-time code generation
- Template expressions (`${...}`)
- Loops and conditionals
- Hygiene (avoid name collisions)
- IDE support for macro expansion preview
- Standard library of common macros

---

## Implementation Priorities Summary

### ✅ COMPLETED (December 25-26, 2025)

**Grammar & Parser Features:**
1. ✅ Fragment System with typed parameters, slots, events
2. ✅ Watcher Hooks with debounce, immediate, deep options
3. ✅ Enhanced Type System (unions, generics, interfaces, optionals)
4. ✅ Computed Properties with @computed decorator
5. ✅ Required Attributes validation
6. ✅ Import/Export System (TypeScript + Rust style)
7. ✅ Validation Rules (Zod v4-compatible, 40+ validators)
8. ✅ CSS-in-DSL (scoped/global, Tailwind @apply/@screen)
9. ✅ Action Handler Syntax (`[]` → `{}`)

**Build System:**
1. ✅ Modularized build.rs (5 focused modules)
2. ✅ TypeScript schema synchronization (32 components)
3. ✅ Auto-generated documentation

**Testing:**
- 51+ tests covering all implemented features

### Remaining: Q1-Q2 2026 (Must-Have)
1. ⏳ LSP Implementation (3 weeks)
2. ⏳ Rich Error Messages (1 week)
3. ⏳ Formatter (1 week)
4. ⏳ i18n System (2 weeks)

**Total: ~7 weeks remaining for foundational DX**

### Q3 2026 (Nice-to-Have)
1. CLI Tool Suite (1 week)
2. Testing Utilities (2 weeks)
3. Accessibility Linter (1 week)

**Total: ~4 weeks for tooling**

### Q4 2026+ (Future)
- Plugin System
- Async/Await
- HMR
- Visual Builder
- Performance Hints

---

## Migration Strategy

### From React
```typescript
// React
function UserCard({ user }) {
  const [expanded, setExpanded] = useState(false);
  
  return (
    <div className="card">
      <h2>{user.name}</h2>
      {expanded && <p>{user.bio}</p>}
      <button onClick={() => setExpanded(!expanded)}>
        {expanded ? 'Collapse' : 'Expand'}
      </button>
    </div>
  );
}
```

```orbis
// Orbis (with fragments)
fragment UserCard(user: User) {
    <Card className="card">
        <Heading level="2" content={user.name} />
        if state.expanded {
            <Text content={user.bio} />
        }
        <Button 
            label={state.expanded ? "Collapse" : "Expand"}
            @click => [state.expanded = !state.expanded]
        />
    </Card>
}
```

**Migration tool**: `orbis migrate --from=react UserCard.jsx`

### From Vue
```vue
<!-- Vue -->
<template>
  <div class="card">
    <h2>{{ user.name }}</h2>
    <p v-if="expanded">{{ user.bio }}</p>
    <button @click="expanded = !expanded">
      {{ expanded ? 'Collapse' : 'Expand' }}
    </button>
  </div>
</template>

<script>
export default {
  props: ['user'],
  data() {
    return { expanded: false }
  }
}
</script>
```

→ Very similar mapping to Orbis

---

## Principles to Maintain

### ✅ Keep
1. **JSX-like syntax** - Familiar to React/Vue developers
2. **Expression-based** - Simple, not Turing-complete
3. **Component whitelisting** - Security and clarity
4. **Strong typing** - Catch errors at build time
5. **Documentation-first** - Auto-generate from definitions

### ❌ Avoid
1. **Turing-complete logic** - Not a general-purpose language
2. **Complex class hierarchies** - Keep flat and simple
3. **Magic behavior** - Explicit over implicit
4. **Vendor lock-in** - Use web standards where possible
5. **Bikeshedding** - Formatter/linter enforce consistency

---

## Success Metrics

1. **Adoption**: 50+ plugins using DSL by Q2 2026
2. **DX Score**: >90% developer satisfaction (survey)
3. **Error Recovery**: <30s from error to fix with LSP
4. **Migration Time**: React component → Orbis in <5 minutes
5. **Documentation**: 100% component coverage with examples
6. **Test Coverage**: >90% parser + LSP coverage
7. **Performance**: Parse 1000-line file in <10ms

---

## Conclusion

The Orbis DSL has a solid foundation with **23 components**, **whitelisting**, **strong typing**, and **documentation generation**. The roadmap above focuses on **developer experience** through:

1. **IDE integration** (LSP) - Most impactful
2. **Better errors** - Reduce frustration
3. **Modularity** (fragments + imports) - Enable scale
4. **Type safety** (enhanced types + validation) - Prevent bugs
5. **Tooling** (formatter, CLI, tests) - Streamline workflow

These improvements make Orbis DSL competitive with React/Vue for plugin development while maintaining **security** and **simplicity**.

**Next Steps**:
1. Validate priorities with plugin developers (user research)
2. Prototype LSP with basic autocomplete (2-week spike)
3. Gather feedback on fragment syntax (RFC)
4. Build formatter MVP (reference implementation)

---

*This roadmap is a living document. Update quarterly based on user feedback and ecosystem changes.*
