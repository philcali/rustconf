# Generator Module Refactoring Spec

## Quick Start

This spec guides the refactoring of the rustconf generator module to improve maintainability and reduce complexity before adding new features.

## What's Being Refactored?

The generator module has grown to 1,650 lines with mixed responsibilities. We're splitting it into focused modules:

```
Before:                          After:
generator/mod.rs (1,650 lines)   generator/mod.rs (~200 lines)
+ 6 test files (2,500 lines)     + types.rs
                                 + operations.rs
                                 + notifications.rs
                                 + paths.rs
                                 + tests.rs (consolidated)
```

## Key Improvements

1. **Module Splitting**: Break monolithic file into focused components
2. **Test Consolidation**: Reduce 6+ test files to 3 organized files
3. **Formatting Integration**: Use existing `formatting.rs` abstractions
4. **Pattern Extraction**: Remove CRUD operation duplication (~40% reduction)
5. **Visitor Pattern**: Simplify parser validation (~30% reduction)

## How to Execute

### Option 1: Run All Tasks (Recommended for Automation)

```bash
# Let Kiro execute all tasks in sequence
kiro "Execute all tasks in the generator-refactoring spec"
```

### Option 2: Execute Phase by Phase

```bash
# Phase 1: Module Splitting
kiro "Execute Phase 1 tasks from generator-refactoring spec"

# Phase 2: Test Consolidation
kiro "Execute Phase 2 tasks from generator-refactoring spec"

# ... and so on
```

### Option 3: Manual Execution

Follow the tasks in `tasks.md` manually, checking off each item as you complete it.

## Files in This Spec

- **requirements.md**: User stories and acceptance criteria
- **design.md**: Technical architecture and implementation details
- **tasks.md**: Step-by-step implementation checklist (34 tasks across 6 phases)
- **README.md**: This file - quick reference guide

## Success Criteria

- ✅ All existing tests pass
- ✅ Generated code output is identical
- ✅ Public API unchanged (no breaking changes)
- ✅ `generator/mod.rs` reduced from 1,650 to ~200 lines
- ✅ Test files reduced from 6+ to 3
- ✅ Code duplication reduced by 30-40%

## Phases Overview

### Phase 1: Module Splitting (Tasks 1-6)
Extract type, path, operations, and notification generation into separate modules.
**Time: 4-6 hours**

### Phase 2: Test Consolidation (Tasks 7-13)
Merge related test files into organized submodules.
**Time: 2-3 hours**

### Phase 3: Formatting Integration (Tasks 14-17)
Use `formatting.rs` abstractions instead of manual string building.
**Time: 3-4 hours**

### Phase 4: CRUD Pattern Extraction (Tasks 18-22)
Create generic CRUD operation generator to eliminate duplication.
**Time: 3-4 hours**

### Phase 5: Parser Visitor Pattern (Tasks 23-28)
Implement visitor pattern for data node traversal in parser.
**Time: 4-5 hours**

### Phase 6: Validation Collection (Tasks 29-30)
Apply visitor pattern to validation type collection.
**Time: 2-3 hours**

**Total Estimated Time: 18-25 hours**

## Testing Strategy

After each phase:
1. Run `cargo test` - all tests must pass
2. Run `cargo clippy` - no new warnings
3. Compare generated code output - should be identical
4. Commit the phase independently

## Rollback Strategy

Each phase is independently valuable and can be committed separately. If issues arise:
1. Revert the specific phase
2. Fix the issue
3. Re-apply the phase

## Dependencies

- Existing `formatting.rs` module
- Existing `quote` and `syn` dependencies
- Existing test suite

## Next Steps After Completion

Once this refactoring is complete, you'll have:
- A clean, modular codebase ready for new features
- Clear patterns for adding new code generation
- Reduced cognitive load when navigating the code
- Better test organization for faster development

## Questions?

Refer to:
- `requirements.md` for "why" we're doing this
- `design.md` for "how" we're implementing it
- `tasks.md` for "what" needs to be done

## Getting Started

To begin the refactoring:

```bash
# Review the spec
cat .kiro/specs/generator-refactoring/requirements.md
cat .kiro/specs/generator-refactoring/design.md

# Start with Phase 1
kiro "Start task 1.1 from generator-refactoring spec"
```

Or simply:

```bash
kiro "Execute the generator-refactoring spec"
```

Good luck! 🚀
