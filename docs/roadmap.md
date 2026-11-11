# Roadmap

The implementation targets the published ECMA-262 baseline described in
[design](design.md#scope). The [specification review](spec-review.md) records
implemented clause groups; [coverage](coverage.md) records current constraints.
This roadmap tracks capabilities rather than individual patches or test runs.

## Implemented capabilities

- Source representations, lexical goals, Unicode data, syntax and early errors.
- Primitive values, coercion, references, expressions, statements and completions.
- Environments, functions, eval, classes and private elements.
- Ordinary/exotic objects, standard built-ins and reflection.
- RegExp matching and String protocols through the shared backend.
- Buffers, typed arrays, shared memory, Atomics and agents.
- Generators, async functions/iteration, jobs and module graphs.
- Weak references, explicit collection and finalization jobs.
- Embedding and command-line APIs with opt-in resource quotas.
- Pinned upstream conformance fixtures, scope decisions and source provenance.

## Ongoing maintenance

- Fix specification and regression-test failures as they are found.
- Keep host policies, native constraints and specification interpretations explicit.
- Review new Unicode data and upstream tests against the published baseline.
- Adopt later published editions through an explicit baseline change.
- Keep crate boundaries, test commands and documentation focused and maintainable.

Later proposals, browser APIs and Node.js compatibility are separate from this
roadmap. Conformance conclusions retain the qualifications in the specification
review; a completed capability is not an unconditional compatibility guarantee.
