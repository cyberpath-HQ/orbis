---
sidebar_position: 100
title: Changelog
description: Version history and release notes
---

## Changelog

All notable changes to Orbis are documented here.

## [1.0.0] - 2025-01-XX

### Added

- Initial stable release
- Schema-driven UI system with 35+ components
- Plugin system with WASM sandboxing
- Standalone mode with SQLite database
- Client-server mode with PostgreSQL
- JWT-based authentication
- Role-based access control
- Hot module reloading for plugins
- Dark mode support
- Accessibility features (ARIA, keyboard navigation)

### Components

- **Layout:** Container, Flex, Grid, Spacer, Divider, Section
- **Typography:** Text, Heading
- **Forms:** Form, Field (15+ field types)
- **Data Display:** Table, List, Card, StatCard, Badge, Avatar, Image, Chart, DataDisplay
- **Feedback:** Alert, Progress, Skeleton, LoadingOverlay, EmptyState
- **Navigation:** Button, Link, Tabs, Breadcrumb, Dropdown, PageHeader
- **Overlays:** Modal, Tooltip
- **Advanced:** Conditional, Loop, Accordion, Icon, Fragment, Slot, Custom

### Actions

- `updateState` - State management
- `callApi` - API calls with full lifecycle hooks
- `navigate` - Client-side routing
- `showToast` - Toast notifications
- `showDialog` / `closeDialog` - Modal dialogs
- `validateForm` / `resetForm` - Form management
- `setLoading` - Loading states
- `download` - File downloads
- `copy` - Clipboard operations
- `openUrl` - External URLs
- `emit` - Custom events
- `debouncedAction` - Debounced execution
- `conditional` - Conditional execution
- `sequence` - Sequential execution

### Security

- Argon2id password hashing
- JWT token authentication
- CSRF protection
- Security headers support
- TLS/SSL support

---

## Versioning

Orbis follows [Semantic Versioning](https://semver.org/):

- **MAJOR:** Breaking changes
- **MINOR:** New features (backward compatible)
- **PATCH:** Bug fixes (backward compatible)

## Upgrade Guide

### From Beta to 1.0

1. Backup your database
2. Update plugin manifests to v1 format
3. Review breaking changes
4. Test in staging environment
5. Deploy to production

### Migration Notes

- Schema format is stable from 1.0
- Plugin API is backward compatible within minor versions
- Database migrations run automatically (configurable)

## Reporting Issues

Found a bug? Please report it on [GitHub Issues](https://github.com/orbis/orbis/issues).

Include:

- Orbis version
- Operating system
- Steps to reproduce
- Expected vs actual behavior
- Relevant logs

## Contributing

See [CONTRIBUTING.md](https://github.com/orbis/orbis/blob/main/CONTRIBUTING.md) for guidelines.
