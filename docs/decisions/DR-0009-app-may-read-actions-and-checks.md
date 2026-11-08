<!--
SPDX-FileCopyrightText: Kreqo contributors
SPDX-License-Identifier: CC-BY-4.0
-->

# DR-0009: The agent's App may read actions and checks

Status: accepted   Date: 2026-11-08   Scope: org

## What you are approving

The agent's GitHub App may hold read access to actions and checks, so that a session can read why its pull request was refused. It never holds write access to either. This narrows DR-0008 row 2 to write access.

## Context

Origin: preparing step 6. The forge serves the review artifact, job logs and check results only to an identity with actions or checks read access. DR-0008 row 2 barred both entirely, so a session couldn't see the AI review that refused its pull request.

Evidence: judgment. The repository is public, so its logs and check results are public already, and read access gives the App nothing an outsider couldn't see. Step 6's gating trial (T2) confirms that the session can read the review.

## Options

1. **Read access only (recommended):** sessions read findings directly. Reversibility: easy.
2. **The gate posts the review as a comment:** row 2 stays as it is, at the cost of more gated workflow code. Reversibility: easy.
3. **The founder pastes findings in by hand:** no change, but every refused pull request waits for the founder. Reversibility: easy.

## Recommendation and why

Option 1. It costs no security that the public repository doesn't already give away, and it keeps refused pull requests from waiting on the founder.

## Rollout / undo plan

The App's manifest (`conventions/forge/agent-app-manifest.json`) and the token script's permission check allow `actions: read` and `checks: read`, and refuse any write. On acceptance, DR-0008's status gains "row 2 superseded by DR-0009". Undo: a later record removes read access.
