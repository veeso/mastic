
# Milestone 3 - SNS Launch

**Duration:** 1 month

**Goal:** Launch Mastic on the Service Nervous System (SNS) to establish fully
decentralised, community-driven governance. Token holders can vote on
proposals for moderation, policy changes, canister upgrades, and treasury
management.

**User Stories:** None (infrastructure milestone)

**Prerequisites:** Milestone 2 completed, all canisters deployed and stable on
mainnet.

## Work Items

### WI-3.1: Prepare SNS configuration (`sns_init.yaml`)

**Description:** Define the SNS initialization parameters, token distribution,
and governance model for the Mastic DAO in the canonical `sns_init.yaml`
configuration file.

**What should be done:**

- Create `sns_init.yaml` with all required parameters:
  - **Project metadata:** name, description, logo, URL
  - **NNS proposal text:** title, forum URL, summary
  - **Fallback controllers:** principal IDs that regain control if the swap
    fails (critical — without these the dapp becomes uncontrollable)
  - **Dapp canisters:** Directory and Federation canister IDs to
    decentralize (User Canisters are managed by Directory, not directly
    by SNS Root)
  - **Token configuration:** name, symbol, transaction fee, logo
  - **Governance parameters:**
    - Proposal rejection fee
    - Initial voting period (>= 4 days recommended)
    - Maximum wait-for-quiet deadline extension
    - Minimum neuron creation stake
    - Minimum dissolve delay for voting (>= 1 month)
    - Dissolve delay bonus (duration + percentage)
    - Age bonus (duration + percentage)
    - Reward rate (initial, final, transition duration)
  - **Token distribution:**
    - Developer neurons with dissolve delay (>= 6 months) and vesting
      period (12-48 months) to signal long-term commitment
    - Seed investor neurons (if any) with vesting
    - Treasury allocation (DAO-controlled)
    - Swap allocation (sold during decentralization swap)
    - Total supply must equal the sum of all allocations
  - **Decentralization swap parameters:**
    - `minimum_participants` (100-200 recommended, not too high)
    - Minimum/maximum direct participation ICP
    - Per-participant minimum/maximum ICP
    - Duration (3-7 days recommended)
    - Neurons fund participation (true/false)
    - Vesting schedule for swap neurons (events + interval)
    - Confirmation text (legal disclaimer)
    - Restricted countries list
- Validate the configuration with `dfx sns init-config-file validate`
- Document the governance model and tokenomics rationale in
  `docs/src/governance.md`
- Study successful SNS launches (OpenChat, Hot or Not, Kinic) for
  parameter ranges the NNS community accepts

**Acceptance Criteria:**

- `sns_init.yaml` passes `dfx sns init-config-file validate`
- Token distribution adds up to the total supply exactly
- Developer neurons have non-zero dissolve delay and vesting period
- Governance parameters are reasonable (voting period >= 4 days, quorum
  defined, rejection fee set)
- Fallback controller principals are defined
- Documentation explains tokenomics and governance model clearly

### WI-3.2: Implement SNS-compatible canister upgrade path

**Description:** Ensure the Directory and Federation canisters can be upgraded
through SNS proposals, and that User Canisters (dynamically created) can be
batch-upgraded by the Directory Canister.

**What should be done:**

- Verify `pre_upgrade` and `post_upgrade` hooks correctly serialize and
  deserialize all state for Directory and Federation canisters
- For User Canisters: verify `wasm-dbms` stable memory survives upgrades
- Implement `set_sns_governance` on the Directory Canister:
  - Accept a principal ID for the SNS governance canister
  - Only callable by canister controllers (before SNS launch) or by
    the already-set governance principal
  - Can only be set once (trap on second call)
- Implement a `require_governance(caller)` guard for governance-gated
  methods
- Implement `upgrade_user_canisters` method on the Directory Canister:
  - Accept new User Canister WASM as argument
  - Callable only by SNS governance (via proposal)
  - Iterate over all registered User Canisters
  - Call `install_code` with mode `Upgrade` for each
  - Track progress and report failures (individual failures must not
    block the batch)
- Test upgrade paths with state preservation

**Acceptance Criteria:**

- All canister state survives an upgrade cycle
- `set_sns_governance` can only be called once by a controller
- Governance-gated methods reject unauthorized callers
- The Directory Canister can batch-upgrade all User Canisters
- Upgrade failures for individual User Canisters do not block the batch
- Integration test: deploy, populate state, upgrade, verify state preserved

### WI-3.3: Implement SNS-governed moderation proposals

**Description:** Transition moderation actions from direct moderator calls to
SNS proposal-based governance. The SNS governance canister becomes the sole
authority for moderation.

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
- Restrict existing direct `add_moderator`, `remove_moderator`, and
  `suspend` methods to the SNS governance canister principal only (no
  longer callable by individual moderators directly)

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

### WI-3.5: SNS testflight on local replica and mainnet

**Description:** Deploy a testflight (mock) SNS to validate the full
governance flow before submitting the real NNS proposal. This catches
configuration issues early, before they are visible to the NNS community.

**What should be done:**

- **Local testflight:**
  - Deploy NNS canisters locally using `sns-testing` repo tooling
  - Deploy Mastic canisters locally
  - Deploy a local testflight SNS using `sns_init.yaml`
  - Test: submit a proposal to upgrade the Directory Canister, vote,
    verify upgrade succeeds
  - Test: submit a moderation proposal, vote, verify action is applied
  - Test: batch-upgrade User Canisters via proposal
- **Mainnet testflight:**
  - Deploy a mock SNS on mainnet (does not run a real swap)
  - Verify governance flows: proposal submission, voting, execution
  - Verify canister upgrade path end-to-end
  - Verify User Canister batch upgrade

**Acceptance Criteria:**

- Local testflight passes all governance flow tests
- Mainnet testflight demonstrates working proposal → vote → execute cycle
- No issues discovered that would block the real launch

### WI-3.6: SNS deployment and decentralization swap

**Description:** Submit the NNS proposal, transfer canister control to
SNS Root, and execute the decentralization swap. This follows the 11-stage
SNS launch process.

**What should be done:**

- **Pre-submission:**
  - Add NNS Root (`r7inp-6aaaa-aaaaa-aaabq-cai`) as co-controller of
    the Directory and Federation canisters using
    `dfx sns prepare-canisters add-nns-root`
  - Final validation of `sns_init.yaml`
  - Call `set_sns_governance` on the Directory Canister with the
    expected SNS governance canister ID (set after SNS-W deploys it)
- **Submit NNS proposal:**
  - `dfx sns propose --network ic --neuron $NEURON_ID sns_init.yaml`
  - This is irreversible once submitted — double-check all parameters
- **During swap (3-7 days):**
  - Monitor swap participation and ICP raised
  - Note: six governance proposal types are restricted during the swap
    (`ManageNervousSystemParameters`, `TransferSnsTreasuryFunds`,
    `MintSnsTokens`, `UpgradeSnsControlledCanister`,
    `RegisterDappCanisters`, `DeregisterDappCanisters`) — do not plan
    operations requiring these
- **Post-swap finalization:**
  - Verify all canisters are controlled solely by SNS Root
  - Verify token holders can submit and vote on proposals
  - Document the post-swap governance workflow (how to submit proposals,
    vote, and execute upgrades)

**Acceptance Criteria:**

- The NNS proposal is submitted and adopted by the community
- SNS-W deploys all SNS canisters (Governance, Ledger, Root, Swap,
  Index, Archive)
- SNS Root becomes sole controller of Directory and Federation canisters
- The decentralization swap completes successfully (meets minimum
  participants and ICP thresholds)
- Token holders can submit and vote on proposals
- Post-swap documentation is published

### WI-3.7: Integration tests for SNS governance flows

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
- **Test `set_sns_governance`:** Verify it can only be set once and only by
  controllers

**Acceptance Criteria:**

- All governance flows pass as integration tests
- Tests simulate SNS governance canister calls
- Each test is independent and can run in isolation
- Tests run in CI via `just integration_test`
