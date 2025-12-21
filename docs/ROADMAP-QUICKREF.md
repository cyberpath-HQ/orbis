# Roadmap Quick Reference

Quick commands and tips for managing the Orbis roadmap.

## Quick Commands

```bash
# Manually trigger roadmap update
./.github/workflows/trigger-roadmap-update.sh

# Or with gh CLI
gh workflow run update-roadmap.yml

# Watch workflow progress
gh run watch

# View recent workflow runs
gh run list --workflow=update-roadmap.yml --limit 5
```

## Feature Status Meanings

| Status | Meaning |
|--------|---------|
| Planned | No issues or PRs exist yet |
| In Progress | Has open issues or pull requests |
| Completed | All related issues closed/PRs merged |

## Adding a New Feature

1. Edit `docs/src/docs/roadmap.mdx` - add feature description
2. Edit `.github/workflows/update-roadmap.yml` - add to features array
3. Trigger update: `./.github/workflows/trigger-roadmap-update.sh`

## File Locations

```
docs/src/docs/roadmap.mdx              # Main roadmap document
.github/workflows/update-roadmap.yml   # Auto-update workflow
.github/workflows/trigger-roadmap-update.sh  # Manual trigger script
docs/ROADMAP-MANAGEMENT.md             # Complete guide
```

## Workflow Schedule

- **Daily**: 2 AM UTC (automatic)
- **On Events**: Issue/PR open, close, label change
- **Manual**: Run trigger script anytime

## Common Issues

### Table not updating?
Check workflow logs: https://github.com/cyberpath-HQ/orbis/actions/workflows/update-roadmap.yml

### Feature missing from table?
Verify it's in the workflow's `features` array and trigger manual update

### Wrong status?
Check search query matches issue titles/descriptions

## Priority Guidelines

- **High**: Security, core features, critical bugs, CI/CD
- **Medium**: Performance, docs, DX improvements, refactoring  
- **Low**: Experimental features, nice-to-haves, future vision

## Creating Roadmap Issues

Good issue titles (include keywords from search queries):
- ✅ "Add Plugin Permissions Modal UI"
- ✅ "Implement Encrypted Configuration Files"
- ✅ "Create Pest-based DSL Parser"

Poor issue titles:
- ❌ "Fix bug"
- ❌ "Improvement"
- ❌ "Update code"

## Support

- Questions: [GitHub Discussions](https://github.com/cyberpath-HQ/orbis/discussions)
- Bug reports: Create issue with `roadmap` label
- Full guide: See `docs/ROADMAP-MANAGEMENT.md`
