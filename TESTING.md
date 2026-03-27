# Testing Guidelines

## Overview

This project uses a three-tier testing strategy:

| Layer | Tool | Location | Purpose |
|-------|------|----------|---------|
| Unit | Jest + ts-jest | `*/src/**/*.test.ts(x)` | Functions, hooks, slices |
| Integration | Jest + React Testing Library | `components/src/**/*.test.tsx` | Component behaviour |
| E2E | Cypress | `components/cypress/e2e/` | Full user flows |

## Running Tests

```bash
# All unit/integration tests across all packages
npm test

# With coverage reports
npm run test:coverage

# E2E tests (requires app running on localhost:3000)
npm run test:e2e

# Watch mode (inside a package)
cd components && npm run test:watch
```

## Coverage Thresholds

All packages enforce **70% minimum** on branches, functions, lines, and statements.
Coverage reports are written to `coverage/` in each package directory.

## Unit Tests

- One test file per source file, co-located: `Button.tsx` → `Button.test.tsx`
- Use `describe` to group related cases; use `it`/`test` for individual assertions
- Mock external dependencies with `jest.fn()` or `jest.mock()`
- Keep tests independent — no shared mutable state between tests

```ts
// Good
it('disables button when loading', () => {
  render(<Button loading>Save</Button>);
  expect(screen.getByText('Save')).toBeDisabled();
});
```

## Integration Tests (React Testing Library)

- Test behaviour, not implementation details
- Query by accessible roles/labels (`getByRole`, `getByLabelText`) over test IDs
- Use `userEvent` over `fireEvent` for realistic interactions
- Assert on what the user sees, not internal state

```ts
import userEvent from '@testing-library/user-event';

it('submits form with valid data', async () => {
  const onSubmit = jest.fn();
  render(<TradeForm onSubmit={onSubmit} />);
  await userEvent.type(screen.getByLabelText('Amount (USDC)'), '100');
  await userEvent.click(screen.getByRole('button', { name: 'Create Trade' }));
  expect(onSubmit).toHaveBeenCalledWith(expect.objectContaining({ amount: '100' }));
});
```

## E2E Tests (Cypress)

- Located in `components/cypress/e2e/`
- Use `data-testid` attributes for selectors that are stable across refactors
- Each spec file covers one user flow (e.g., `trade-flow.cy.ts`, `dispute-flow.cy.ts`)
- Run against a locally running app: `npm run test:e2e:open` for interactive mode

```ts
it('creates a new trade', () => {
  cy.visit('/');
  cy.get('[data-testid="create-trade-btn"]').click();
  cy.get('[data-testid="amount"]').type('100');
  cy.get('[data-testid="submit-trade"]').click();
  cy.get('[data-testid="trade-status"]').should('contain', 'created');
});
```

## What to Test

**Do test:**
- Happy path and error/edge cases for business logic
- Component rendering with different prop combinations
- Form validation and submission
- Redux slice reducers and selectors
- Security utilities (sanitization, validation)

**Don't test:**
- Third-party library internals
- Implementation details (internal state, private methods)
- Styling/CSS classes unless they affect behaviour

## Adding New Tests

1. Create `<ComponentName>.test.tsx` alongside the source file
2. Import from `@testing-library/react` for React components
3. Run `npm run test:coverage` to verify thresholds are met
4. For new user flows, add a Cypress spec in `components/cypress/e2e/`


---

## Accessibility Testing

Located in `testing/accessibility/`. Automated axe-core tests + manual procedures.

```bash
# Run automated a11y tests
npm run test:a11y

# Generate HTML report
cd testing/accessibility && node scripts/generate-a11y-report.js
```

See `testing/accessibility/MANUAL_TESTING_PROCEDURES.md` for screen reader and keyboard testing procedures.
See `testing/accessibility/WCAG_CHECKLIST.md` for the full WCAG 2.1 AA compliance checklist.

---

## Testing Documentation

Full testing documentation is in `docs/testing/`:

| Document | Purpose |
|----------|---------|
| `TEST_STRATEGY.md` | Overall testing philosophy, pyramid, quality gates |
| `TEST_PLAN_TEMPLATE.md` | Template for feature-level test plans |
| `TEST_CASE_DOCUMENTATION.md` | Canonical test case catalog with AC mapping |
| `TESTING_GUIDELINES.md` | Practical guidelines for writing tests |
