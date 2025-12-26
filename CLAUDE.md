# DBRunner - AI Assistant Documentation

This file contains comprehensive information about the DBRunner project to help AI assistants and developers understand the codebase quickly.

## Project Overview

DBRunner is a desktop application built with Tauri (Rust + Svelte) that provides a simple GUI for managing local database containers using Docker Compose. It allows developers to quickly spin up PostgreSQL, MySQL, MongoDB, and Redis instances for local development without managing Docker Compose files manually.

## Tech Stack

### Backend
- **Tauri**: Desktop application framework
- **Rust**: Backend logic and system integration
- **Docker & Docker Compose**: Container orchestration
- **serde**: JSON serialization/deserialization

### Frontend
- **Svelte 5**: UI framework (using runes: `$state`, `$props`, `$effect`)
- **SvelteKit**: Application framework with adapter-static for SPA
- **Vite 6**: Build tool and dev server
- **Tailwind CSS 3.4**: Utility-first styling with dark mode support
- **TypeScript**: Type safety

## Project Structure

```
dbrunner/
├── src/                          # Frontend Svelte code
│   ├── routes/
│   │   └── +page.svelte         # Main dashboard
│   └── lib/
│       └── components/
│           ├── DatabaseCard.svelte      # Individual DB card
│           ├── ConnectionString.svelte  # Connection info display
│           ├── LogsModal.svelte         # Container logs viewer
│           └── ThemeToggle.svelte       # Dark/light mode
├── src-tauri/
│   └── src/
│       └── lib.rs               # Main Rust backend logic
└── ~/.config/dbrunner/
    └── config.json              # User configuration storage
```

## Key Architecture Concepts

### Configuration System

**Location**: `~/.config/dbrunner/config.json`

**Structure**:
```json
{
  "volume_paths": {
    "postgresql": "/custom/path/to/data",
    "mysql": "/another/path"
  },
  "image_tags": {
    "postgresql": "15-alpine",
    "redis": "7.2-alpine"
  }
}
```

**Backend Config Struct** (src-tauri/src/lib.rs:30-35):
```rust
#[derive(Serialize, Deserialize, Clone, Default)]
struct Config {
    volume_paths: HashMap<String, String>,
    #[serde(default)]  // Important for backward compatibility
    image_tags: HashMap<String, String>,
}
```

**Important Notes**:
- Config is loaded once and cached in `Mutex<Option<Config>>`
- Empty values in config trigger fallback to defaults
- `#[serde(default)]` ensures backward compatibility when adding new fields

### Database Information Flow

1. **Frontend calls** `list_databases()` command
2. **Backend loads config** and constructs `DatabaseInfo` structs
3. **Image tags are resolved**: Custom from config or default constants
4. **Status is checked** separately via `get_database_status()`
5. **Frontend receives** updated database list with current state

### Docker Compose Generation

**Key Function**: `generate_docker_compose()` (src-tauri/src/lib.rs:252-385)

**Process**:
1. Gets image via `get_database_image()` helper (respects custom tags)
2. Matches database name to configuration (ports, env vars, health checks)
3. Conditionally includes environment section (empty for Redis)
4. Generates volume names with explicit mapping (postgresql→postgres_data)
5. Writes to temp file: `/tmp/dbrunner-{dbname}.yml`
6. Executes: `docker compose -f /tmp/dbrunner-{dbname}.yml up -d`

**Important Gotchas**:
- Redis has no environment variables, so env section must be completely omitted (not empty)
- Volume names must use explicit mapping, not string replacement
- Docker Compose version field is obsolete but harmless (warning only)

### Image Tag Configuration

**Default Tags** (src-tauri/src/lib.rs:8-11):
```rust
const DEFAULT_POSTGRES_TAG: &str = "18-alpine";
const DEFAULT_MYSQL_TAG: &str = "8.0";
const DEFAULT_MONGODB_TAG: &str = "8";
const DEFAULT_REDIS_TAG: &str = "8-alpine";
```

**Base Image Mapping**:
- `postgresql` → `postgres` (Docker Hub name)
- `mysql` → `mysql`
- `mongodb` → `mongo` (Docker Hub name)
- `redis` → `redis`

**Tag Resolution Logic** (`get_database_image()` at line 72):
1. Check config for custom tag
2. Fall back to default constant if not found
3. Combine: `{base_image}:{tag}` (e.g., `postgres:18-alpine`)

**Clearing Custom Tags**:
- Saving empty string removes entry from config
- Backend returns: "Reset {DatabaseName} to default image tag"
- Frontend refetches to display default

### Volume Name Generation

**Critical Function** (src-tauri/src/lib.rs:316-322):
```rust
let volume_name = match db_name {
    "postgresql" => "postgres",
    "mysql" => "mysql",
    "mongodb" => "mongodb",
    "redis" => "redis",
    _ => db_name,
};
```

**Why Explicit Mapping**:
- Previous code used `.replace("sql", "")` which caused bugs
- "postgresql" → "postgre_data" ❌ (missing directory error)
- "mysql" → "my_data" ❌ (incorrect)
- Explicit mapping ensures correctness

## Tauri Commands (Backend → Frontend API)

**Database Operations**:
- `list_databases() -> Vec<DatabaseInfo>`: Get all databases with config
- `start_database(dbName: String) -> CommandResult`: Start container
- `stop_database(dbName: String) -> CommandResult`: Stop container
- `get_database_status(dbName: String) -> String`: Check if running

**Configuration**:
- `set_volume_path(dbName: String, path: String) -> CommandResult`
- `get_volume_path(dbName: String) -> Option<String>`
- `set_image_tag(dbName: String, tag: String) -> CommandResult`
- `get_image_tag(dbName: String) -> Option<String>`

**Utilities**:
- `get_container_logs(dbName: String, tailLines: Option<usize>) -> Result<String>`
- `generate_connection_strings(dbName: String, port: u16) -> Result<HashMap>`

**Command Result Pattern**:
```rust
struct CommandResult {
    success: bool,
    message: String,  // User-facing message for UI display
}
```

## Frontend Patterns

### Component Structure (DatabaseCard.svelte)

**State Management**:
```svelte
let loading = $state(false);
let editingVolumePath = $state(false);
let newVolumePath = $state("");
let editingImageTag = $state(false);
let newImageTag = $state("");
let showConnectionString = $state(false);
```

**Props**:
```svelte
let {
  db = $bindable<DatabaseInfo>(),
  onMessage,
  onOpenLogs,
  onCopy,
} = $props();
```

**Key Patterns**:
1. **Inline Editing**: Click edit (✏️) → show input → save/cancel
2. **Loading States**: Disable all interactive elements during operations
3. **Conditional Rendering**: `{#if}/{:else}` blocks for different states
4. **Two-way Binding**: `bind:value` for inputs, `$bindable` for props

### Loading State Implementation

**Visual Feedback**:
- Animated spinner SVG (Tailwind `animate-spin`)
- Text changes: "▶ Start" → "Starting..." with spinner
- Button disabled with `opacity-60`
- Edit buttons disabled with helpful tooltips

**What Gets Disabled During Loading**:
- Start/Stop buttons
- Image tag edit button
- Volume path edit button

## Environment Variables & Credentials

**PostgreSQL**:
```yaml
POSTGRES_USER: postgres
POSTGRES_PASSWORD: postgres
POSTGRES_DB: devdb
```

**MySQL**:
```yaml
MYSQL_ROOT_PASSWORD: root
MYSQL_DATABASE: devdb
MYSQL_USER: mysql
MYSQL_PASSWORD: mysql
```

**MongoDB**:
```yaml
MONGO_INITDB_ROOT_USERNAME: admin
MONGO_INITDB_ROOT_PASSWORD: admin
MONGO_INITDB_DATABASE: devdb
```

**Redis**: No authentication (none needed for local dev)

## Container Naming Convention

- PostgreSQL: `dbrunner-postgres`
- MySQL: `dbrunner-mysql`
- MongoDB: `dbrunner-mongodb`
- Redis: `dbrunner-redis`

## Health Checks

**PostgreSQL**: `pg_isready -U postgres`
**MySQL**: `mysqladmin ping -h localhost -u root -proot`
**MongoDB**: `mongosh --eval "db.adminCommand('ping')"`
**Redis**: `redis-cli ping`

All use: 10s interval, 5s timeout, 5 retries

## Development Workflow

**Running the app**:
```bash
pnpm tauri dev
```

**Build process**:
1. Vite builds frontend (Svelte → JS)
2. Cargo compiles Rust backend
3. Tauri bundles into desktop app

**Hot reload**:
- Frontend: Automatic via Vite
- Backend: Automatic via cargo-watch

## Common Issues & Solutions

### 1. Redis Environment Error
**Problem**: `services.redis.environment must be a mapping`
**Solution**: Conditionally include environment section only if env_vars is not empty

### 2. PostgreSQL Volume Mount Error
**Problem**: `no such file or directory` for volume mount
**Solution**: Use explicit volume name mapping instead of string replacement

### 3. Empty Tag Validation
**Problem**: Users couldn't revert to defaults
**Solution**: Allow empty tags, remove from config to trigger default fallback

### 4. Confusing Loading States
**Problem**: No visual feedback during long Docker pulls
**Solution**: Add spinner animation and disable all editable controls

## Docker Compose Template Structure

```yaml
version: '3.8'

services:
  {db_name}:
    image: {image}
    container_name: {container_name}
    environment:           # Omitted if empty (Redis)
      {env_vars}
    ports:
      - "{port}"
    volumes:
      - {volume_line}      # Custom path or named volume
    restart: unless-stopped
    healthcheck:
      {health_check}

volumes:                   # Omitted if using custom path
  {volume_name}_data:
    driver: local
```

## Validation Rules

**Image Tags**:
- Cannot contain `:` (colon) - indicates full image string
- Cannot contain `/` (slash) - indicates registry path
- Max length: 100 characters
- Empty is allowed (resets to default)

**Volume Paths**:
- Must exist on filesystem (validated by Rust)
- Can be empty (triggers Docker named volume)

## UI/UX Patterns

### Dark Mode
- Tailwind class-based: `dark:bg-gray-800`
- Persisted preference (handled by ThemeToggle component)
- Applies to all components consistently

### Responsive Design
- Mobile: 1 column grid
- Tablet (md): 2 columns
- Desktop (lg): 3 columns
- Grid class: `grid-cols-1 md:grid-cols-2 lg:grid-cols-3`

### Status Badge Colors
- Running: Green (`bg-green-100 text-green-800`)
- Stopped: Red (`bg-red-100 text-red-800`)

### Button States
- Primary action (Start): Blue
- Destructive (Stop): Red
- Secondary (Logs): Orange
- Edit actions: Green (Save), Gray (Cancel)

## Connection String Generation

Provides multiple formats for each database:
- `standard_uri`: Standard connection URI
- `jdbc`: JDBC connection string (or N/A)
- `host`, `port`, `user`, `password`, `database`: Individual components

**Example (PostgreSQL)**:
```
standard_uri: postgresql://postgres:postgres@localhost:5432/devdb
jdbc: jdbc:postgresql://localhost:5432/devdb
host: localhost
port: 5432
user: postgres
password: postgres
database: devdb
```

## File Locations

**Temporary Docker Compose Files**:
- Path: `/tmp/dbrunner-{dbname}.yml`
- Created on start, optionally cleaned on stop
- One per database type

**Config File**:
- Linux: `~/.config/dbrunner/config.json`
- macOS: `~/Library/Application Support/dbrunner/config.json`
- Windows: `%APPDATA%\dbrunner\config.json`

## Future Considerations

**Potential Enhancements**:
- Custom environment variables per database
- Port configuration (currently hardcoded)
- Multiple instances of same database type
- Export/import configurations
- Database backup/restore tools
- Connection testing before container start
- Auto-update for images

**Known Limitations**:
- Single instance per database type
- No custom network configuration
- Fixed credentials (suitable for local dev only)
- No PostgreSQL version-specific extensions
- No data persistence migration tools

## Tips for AI Assistants

1. **Always read existing code** before making changes to understand patterns
2. **Config changes** require `#[serde(default)]` for backward compatibility
3. **Empty states** in YAML require complete omission, not empty values
4. **Volume naming** must use explicit mapping, avoid string manipulation
5. **Frontend state** uses Svelte 5 runes (`$state`, not `let`)
6. **Loading states** should disable all interactive elements
7. **Error messages** should be user-friendly (shown in UI toast)
8. **Database names** are lowercase internally, display names are capitalized
9. **Docker operations** are async - always show loading feedback
10. **Tag validation** happens in backend, UI shows backend messages

## Testing Checklist

When making changes, verify:
- [ ] Backend compiles without warnings
- [ ] Frontend builds without errors
- [ ] Default tags work (fresh install)
- [ ] Custom tags persist across restarts
- [ ] Clearing tags reverts to defaults
- [ ] Volume paths can be set and cleared
- [ ] Start/stop operations work
- [ ] Loading states appear correctly
- [ ] All four databases (PostgreSQL, MySQL, MongoDB, Redis) work
- [ ] Dark mode works correctly
- [ ] Responsive layout works at all breakpoints
- [ ] Docker images pull successfully
- [ ] Containers start with correct health checks
- [ ] Connection strings are accurate
- [ ] Logs display correctly

## Recent Changes (Session Summary)

1. **Added Redis support**: Fourth database option with proper YAML handling
2. **Configurable image tags**: Users can change Docker image versions
3. **Fixed environment section**: Conditionally included only when needed
4. **Fixed volume naming**: Explicit mapping instead of string replacement
5. **Enhanced loading states**: Added spinners and comprehensive disabling
6. **Allow tag reset**: Empty input clears custom tag, reverts to default
7. **Improved UX**: Better tooltips, placeholders, and visual feedback

---

Last updated: 2025-12-26
Project version: Development (not versioned)
Maintained by: Joel (joelseq)
