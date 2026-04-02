
# Milestone 2 - Frontend

**Duration:** 2 months

**Goal:** Build the Mastic web frontend as a React application deployed as an
IC asset canister. The frontend provides Internet Identity authentication and
a Mastodon-like interface covering all user stories from Milestones 0 and 1.
After this milestone, users can interact with the Mastic node entirely through
a browser.

**User Stories:** UC1, UC2, UC3, UC4, UC5, UC6, UC7, UC8, UC9, UC10, UC11,
UC12, UC15, UC16

**Prerequisites:** Milestone 1 completed.

## Work Items

### WI-2.1: Frontend project scaffold & Internet Identity authentication

**Description:** Set up the React project as an IC asset canister with
Internet Identity sign-in, agent configuration, and basic app shell.

**What should be done:**

- Initialize a React project (Vite + TypeScript) under
  `crates/canisters/frontend/`
- Configure `dfx.json` with the frontend asset canister
- Set up `@dfinity/agent` with actor factories generated from the `.did`
  files for Directory, Federation, and User canisters
- Integrate `@dfinity/auth-client` for Internet Identity sign-in/sign-out
- Implement sign-up page: handle input + call `sign_up` on the Directory
  Canister
- Post-auth routing: call `whoami` to resolve the User Canister principal,
  store in app state
- Basic app shell: navigation bar with auth status, client-side routing
  skeleton

**Acceptance Criteria:**

- The frontend deploys as an asset canister via `dfx deploy`
- Users can sign in with Internet Identity and sign out
- New users can sign up by choosing a handle
- After sign-in, the app resolves the User Canister principal and stores it
  in state
- The navigation bar shows the authenticated user's handle
- Routing works for at least `/`, `/sign-up`, and a placeholder home page

### WI-2.2: Feed view & status composer

**Description:** Build the main timeline view with paginated feed and a
status composer for publishing new statuses.

**What should be done:**

- Implement a feed page that calls `read_feed` on the User Canister and
  renders a paginated list of statuses
- Build a status card component displaying: author handle, display name,
  avatar, content, timestamp, like count, boost count
- Implement infinite scroll or "load more" pagination using the cursor
  returned by `read_feed`
- Build a compose form: text input with character count + call
  `publish_status` on the User Canister
- New statuses appear at the top of the feed after publishing

**Acceptance Criteria:**

- The feed displays statuses from followed users and the user's own statuses
- Pagination loads additional statuses without reloading the page
- Status cards show all required fields (author, content, timestamp, counts)
- Publishing a status adds it to the feed
- Empty feed shows a meaningful placeholder message

### WI-2.3: Profile view & management

**Description:** Display user profiles and allow users to edit or delete
their own profile.

**What should be done:**

- Implement a profile page at `/users/{handle}`:
  - Call `get_user` on the Directory Canister to resolve the User Canister
  - Call `get_profile` on the target User Canister
  - Display: handle, display name, bio, avatar, header image,
    follower/following counts
- Own profile: show an edit form for display name, bio, avatar URL, and
  header URL that calls `update_profile` on the User Canister
- Delete account flow: confirmation dialog that calls `delete_profile` on
  the Directory Canister, then redirects to the landing page
- Make author names/avatars in status cards clickable to navigate to the
  author's profile

**Acceptance Criteria:**

- Any user's profile can be viewed by navigating to `/users/{handle}`
- The own profile displays an edit button that opens the edit form
- Updating a field persists the change (verified by reloading the profile)
- Account deletion requires confirmation and redirects after success
- Author links in the feed navigate to the correct profile page

### WI-2.4: Follow, like & boost interactions

**Description:** Implement follow/unfollow, like/unlike, and boost/unboost
UI interactions.

**What should be done:**

- Follow/unfollow button on profile pages:
  - Show "Follow" or "Unfollow" based on current relationship
  - Call `follow_user` / `unfollow_user` on the User Canister
- Followers and following lists on the profile page:
  - Call `get_followers` / `get_following` on the User Canister
  - Render paginated lists of user cards linking to their profiles
- Like button on status cards:
  - Toggle like state, call `like_status` / `undo_like`
  - Update like count optimistically
- Boost button on status cards:
  - Toggle boost state, call `boost_status` / `undo_boost`
  - Update boost count optimistically
- Liked statuses page: call `get_liked` and render the list

**Acceptance Criteria:**

- Follow/unfollow toggles correctly and updates the button state
- Followers and following lists display correct users with pagination
- Like and boost buttons toggle state and update counts immediately
- Undoing a like or boost reverses the action
- The liked statuses page shows all statuses the user has liked

### WI-2.5: User search

**Description:** Implement a search interface for discovering users on the
Mastic node.

**What should be done:**

- Add a search bar in the navigation or a dedicated search page
- Call `search_profiles` on the Directory Canister with the query string
- Display results as user cards (avatar, handle, display name) linking to
  the user's profile
- Implement pagination for search results
- Debounce input to avoid excessive queries

**Acceptance Criteria:**

- Typing a query returns matching users
- Results link to the correct user profiles
- Pagination works when there are many results
- Empty or whitespace-only queries are handled gracefully
- No excessive API calls while the user is still typing

### WI-2.6: Moderation tools

**Description:** Build moderator-specific UI for content and user
moderation. These controls are only visible to users who are moderators.

**What should be done:**

- Detect moderator status (e.g., by checking with the Directory Canister
  whether the current principal is a moderator)
- Show a delete button on any status card when the user is a moderator:
  - Call `delete_status` on the author's User Canister
  - Remove the status from the feed on success
- Show a suspend button on user profiles when the user is a moderator:
  - Confirmation dialog explaining the action
  - Call `suspend` on the Directory Canister
- Moderator management page (accessible from settings or nav):
  - List current moderators
  - Add moderator by principal: call `add_moderator`
  - Remove moderator: call `remove_moderator` (with safeguard against
    removing the last moderator)
- Block user button on profile pages (available to all users):
  - Call `block_user` on the User Canister

**Acceptance Criteria:**

- Non-moderators do not see moderation controls (delete on others' statuses,
  suspend, moderator management)
- Moderators can delete any status and it disappears from the feed
- Moderators can suspend a user, who then cannot interact with the platform
- The moderator list is displayed correctly
- Adding and removing moderators works, with a safeguard against removing
  the last one
- Any user can block another user from their profile page

### WI-2.7: Frontend build pipeline & deployment

**Description:** Integrate the frontend build into the existing `just`
command workflow and CI pipeline.

**What should be done:**

- Add `just build_frontend` command: runs the Vite production build and
  outputs to the asset canister directory
- Add `just dfx_deploy_frontend` command: deploys the asset canister locally
- Update `just build_all` to include the frontend build
- Update `just dfx_deploy_local` to include the frontend canister
- Ensure the frontend build works in CI (install Node.js dependencies,
  run build)
- Add `just test_frontend` command for running frontend unit tests

**Acceptance Criteria:**

- `just build_frontend` produces a production build without errors
- `just dfx_deploy_local` deploys all canisters including the frontend
- The deployed frontend is accessible at the asset canister URL
- CI can build and deploy the frontend
- Frontend unit tests run via `just test_frontend`
