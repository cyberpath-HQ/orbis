# Roadmap Auto-Update Workflow

This GitHub Action automatically updates the roadmap table in `docs/src/docs/roadmap.mdx` with real-time information from GitHub issues and pull requests.

## How It Works

1. **Triggers**:
   - **Daily Schedule**: Runs every day at 2 AM UTC
   - **Manual**: Can be triggered manually via GitHub Actions UI
   - **Issues/PRs**: Automatically runs when issues or PRs are opened, closed, reopened, or labeled

2. **Process**:
   - Searches for issues and PRs matching each roadmap feature
   - Determines feature status based on issue/PR states:
     - **Planned**: No related issues/PRs found
     - **In Progress**: Has open issues or PRs
     - **Completed**: All related issues/PRs are closed/merged
   - Generates markdown table with links to relevant issues/PRs
   - Updates the roadmap file between the auto-generated markers

3. **Output**:
   - Updates `docs/src/docs/roadmap.mdx` with current status
   - Commits changes with message: `chore: update roadmap table with latest issue/PR data [skip ci]`
   - Includes timestamp of last update

## Customization

### Adding New Features

To add a new feature to the auto-update:

1. Add the feature to the roadmap document manually
2. Update the `features` array in `.github/workflows/update-roadmap.yml` with:
   ```javascript
   { 
     priority: 'High|Medium|Low', 
     name: 'Feature Name', 
     query: 'search terms in:title,body' 
   }
   ```

### Search Query Tips

- Use specific keywords that appear in related issues/PRs
- Add `in:title,body` to search both title and body
- Combine multiple terms: `plugin filesystem api`
- Use quotes for exact phrases: `"plugin system"`

### Modifying Status Logic

The workflow determines status based on:
- **Completed**: All issues closed AND at least one PR merged
- **In Progress**: At least one open issue or PR
- **Planned**: No issues or PRs found

To modify this logic, edit the status determination section in the workflow file.

## Manual Testing

To test the workflow:

1. Go to GitHub Actions tab
2. Select "Update Roadmap Table" workflow
3. Click "Run workflow"
4. Check the workflow logs for any errors
5. Review the commit to `roadmap.mdx`

## Troubleshooting

### Table Not Updating

- Check workflow logs for errors
- Verify the auto-generated markers exist in `roadmap.mdx`:
  ```mdx
  {/* AUTO-GENERATED-TABLE-START */}
  {/* AUTO-GENERATED-TABLE-END */}
  ```
- Ensure GitHub token has proper permissions

### Wrong Status Detection

- Review search queries in the workflow
- Check if issue/PR labels or titles match the query
- Consider adding more specific search terms

### Rate Limiting

The workflow respects GitHub API rate limits. If you hit limits:
- Reduce the number of features being tracked
- Limit the frequency of updates
- Use more specific search queries to reduce API calls

## Permissions Required

The workflow needs:
- `contents: write` - To commit changes
- `issues: read` - To search issues
- `pull-requests: read` - To search PRs

These are configured in the workflow file.
