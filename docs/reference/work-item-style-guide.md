---
author: AI Cockpit maintainers
title: "Work Item style guide"
description: "Practical guidance for writing reviewable, evidence-bound Work Items."
audience:
  - adopter
  - contributor
  - maintainer
status: current
authority: canonical
lastVerifiedBy: WI-390-reference-style-guide
capabilityClaims:
  - work_item_style_guidance
---

# Work Item style guide

[简体中文](work-item-style-guide.zh-CN.md) · [日本語](work-item-style-guide.ja.md)

This guide explains how to write a Work Item that a person can review and the
installed Rust Runtime can verify. It is guidance, not a second Contract schema.
The Contract remains the human-owned source of intent, authority, scope,
acceptance, and required evidence.

## State the outcome first

Describe what should be true when the work is complete before describing how
to implement it. If the problem or user benefit was not provided, say so
explicitly. Do not infer motivation, impact, approval, or completion from a
file name, detected technology, or Agent prose.

Use the current Contract fields deliberately:

- `intent` and `goal` describe the human-owned purpose and desired outcome.
- Structured intent may record `businessGoal`, `userGoal`, `problem`,
  `constraints`, `nonGoals`, and `rationale`; each remains optional and must
  stay unknown when the owner did not provide it.
- `intentAlignment` is an optional Summary projection after implementation.
  It records whether the problem, constraints, non-goals, and rationale were
  actually addressed; it does not rewrite the original intent.

## Define the problem and boundaries

Explain why the Work Item exists only when that context is known. Declare
repository-relative `scope` and `outOfScope` before editing. Scope is an
authorization boundary, not a retrospective list of changed files. Keep
non-goals explicit so review can detect accidental expansion.

## Make acceptance observable

Acceptance criteria must be checkable by a person or a declared verification
command. Prefer statements such as “the Contract validator passes” or “the
documented route links resolve” over subjective claims such as “looks good”.
Numbered `A<n>:` criteria can bind Summary evidence; unnumbered criteria remain
readable source-language declarations. The Runtime never invents criteria or
evidence mappings.

## Keep governance decisions human-owned

`authority`, approval, risk acceptance, and any decision to continue after an
unknown belong to the responsible human or an explicitly delegated provider.
The Runtime validates shape, identity, freshness, and evidence; it does not
turn a missing field into permission. A yellow or red preflight is a review
boundary, not an authorization to edit or finish.

## Prefer the smallest sufficient process

Use the existing lifecycle and verification capabilities. Add a field, gate,
or approval step only when it preserves review or audit value. Select the
repository's proportional Light/Standard/Strict profile; a stronger
Verification Tier is not the same thing as stronger Evidence Assurance.

## Record executable verification

Declare the checks that can be run for this repository and execute them with
the installed Runtime and an explicit `--repo`. Verification receipts bind the
Work Item, repository snapshot, and Runtime identity. A declaration alone is
not evidence, and a path that merely exists is not a passing check.

## Extend existing concepts before adding new ones

Check the current Contract, Summary, scenario, evidence, decision, and policy
fields before introducing a new concept. Document the review model first; add
schema only when a deterministic machine check is required. Keep source
language and governance bytes intact—presentation localization must not change
their meaning.

## Object-project inheritance

An adopter repository receives the same reader-facing rules through its
repository-local `.ai/` and Agent adapter, while the shared Runtime remains
outside the project. Repository identity, Contracts, evidence, and knowledge
are isolated per `--repo`. This page does not copy the reference installer's
commands or runtime implementation; it carries forward the applicable
governance semantics in the Rust-native interface.
