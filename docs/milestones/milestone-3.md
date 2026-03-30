---
title: "Milestone 3 - SNS Launch"
layout: page
---

# Milestone 3 - SNS Launch

**Duration:** 1 month

**Goal:** Launch Mastic on the Service Nervous System (SNS) to establish fully
decentralised, community-driven governance. Token holders can vote on
proposals for moderation, policy changes, and canister upgrades.

**User Stories:** None (infrastructure milestone)

**Prerequisites:** Milestone 2 completed.

## Work Items

### WI-3.1: Prepare SNS configuration

**Description:** Define the SNS initialization parameters and token
distribution for the Mastic DAO.

**What should be done:**

- Create the SNS init configuration file (`sns.yml`) with:
  - Token name and symbol
  - Initial token distribution (team, treasury, swap)
  - Governance parameters (proposal thresholds, voting period, minimum stake)
  - Swap parameters (minimum/maximum ICP, duration)
  - Fallback controller principals
- Define the neuron fund participation rules
- Document the governance model in `docs/`

**Acceptance Criteria:**

- `sns.yml` is valid and passes `sns-cli validate`
- Token distribution adds up to the total supply
- Governance parameters are reasonable (voting period >= 24h, quorum defined)
- Documentation explains the governance model clearly

### WI-3.2: Implement SNS-compatible canister upgrade path

**Description:** Ensure all three canisters (Directory, Federation, User) can
be upgraded through SNS proposals.

**What should be done:**

- Verify `pre_upgrade` and `post_upgrade` hooks correctly serialize and
  deserialize all state for each canister
- Ensure the SNS root canister is set as a controller of the Directory and
  Federation canisters
- For User Canisters (dynamically created): implement a mechanism for the
  Directory Canister to upgrade all User Canisters when it receives an
  upgrade proposal
- Add a `upgrade_user_canisters` method to the Directory Canister:
  - Accept new User Canister WASM as argument
  - Iterate over all registered User Canisters
  - Call `install_code` with mode `Upgrade` for each
  - Track progress and report failures
- Test upgrade paths with state preservation

**Acceptance Criteria:**

- All canister state survives an upgrade cycle
- The SNS root canister can trigger upgrades on Directory and Federation
- The Directory Canister can batch-upgrade all User Canisters
- Upgrade failures for individual User Canisters do not block the batch
- Integration test: deploy, populate state, upgrade, verify state preserved

### WI-3.3: Implement SNS-governed moderation proposals

**Description:** Replace direct moderator calls with SNS proposal-based
governance for adding/removing moderators and suspending users.

**What should be done:**

- Implement a generic proposal execution interface on the Directory Canister:
  - Accept proposals from the SNS governance canister
  - Parse proposal payloads to determine the action
- Supported proposal types:
  - `AddModerator`: add a principal to the moderator list
  - `RemoveModerator`: remove a principal from the moderator list
  - `SuspendUser`: suspend a user by handle
  - `UnsuspendUser`: reactivate a suspended user
  - `UpdatePolicy`: update instance moderation policies (e.g., content
    rules text)
- The existing direct `add_moderator`, `remove_moderator`, and `suspend`
  methods should be restricted to the SNS governance canister principal
  (no longer callable by individual moderators directly)

**Acceptance Criteria:**

- Moderation actions can only be executed via SNS proposals
- The Directory Canister correctly parses and executes each proposal type
- Invalid proposal payloads are rejected with a descriptive error
- The SNS governance canister principal is the only authorized caller for
  moderation methods
- Integration test: simulate a proposal execution, verify the action is
  applied

### WI-3.4: Implement UnsuspendUser flow

**Description:** Add the ability to reactivate a suspended user account via
SNS governance.

**What should be done:**

- Implement `unsuspend` method on the Directory Canister:
  - Authorize the caller (SNS governance canister only)
  - Remove the suspended flag from the user record
  - Notify the User Canister to resume operations
  - Optionally send an `Undo(Delete)` or `Update(Person)` activity to
    re-announce the user to followers
- Define `UnsuspendArgs`, `UnsuspendResponse` in the `did` crate

**Acceptance Criteria:**

- A suspended user can be reactivated via the `unsuspend` method
- Only the SNS governance canister can call `unsuspend`
- After unsuspension, the user can interact with their User Canister again
- The user reappears in `search_profiles` results
- Integration test: suspend, then unsuspend, verify the user is active

### WI-3.5: SNS deployment and decentralization swap

**Description:** Deploy Mastic under SNS control and execute the
decentralization swap.

**What should be done:**

- Submit the SNS proposal to the NNS for approval
- Transfer canister control to the SNS root canister
- Configure the decentralization swap parameters
- Verify all canisters are controlled by the SNS after the swap
- Document the post-swap governance workflow (how to submit proposals, vote,
  and execute upgrades)

**Acceptance Criteria:**

- The NNS proposal is submitted and approved
- All canisters (Directory, Federation) are controlled by the SNS root
- The decentralization swap completes successfully
- Token holders can submit and vote on proposals
- Post-swap documentation is published

### WI-3.6: Integration tests for SNS governance flows

**Description:** Write integration tests that validate the SNS governance
integration.

**What should be done:**

- **Test proposal execution:** Simulate an SNS proposal to add a moderator,
  verify the moderator is added
- **Test canister upgrade via SNS:** Simulate an upgrade proposal, verify
  state is preserved
- **Test User Canister batch upgrade:** Upgrade all User Canisters via the
  Directory, verify state
- **Test suspend/unsuspend via proposal:** Simulate suspend and unsuspend
  proposals
- **Test unauthorized access:** Verify that direct moderation calls (not from
  SNS governance) are rejected

**Acceptance Criteria:**

- All governance flows pass as integration tests
- Tests simulate SNS governance canister calls
- Each test is independent and can run in isolation
- Tests run in CI via `just integration_test`
