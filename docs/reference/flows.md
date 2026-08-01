# Aether Flows

## App Runtime

```mermaid
flowchart TD
  Launch[Launch Desktop App] --> Tauri[Tauri Runtime]
  Tauri --> React[React App]
  React --> Query[TanStack Query]
  Query --> SDK[Generated Aether SDK]
  SDK --> Invoke[Tauri Invoke]
  Invoke --> Commands[Rust Commands]
  Commands --> Services[Repositories and Services]
  Services --> DB[(Local Database)]
  Services --> Files[(Media and Model Files)]
```

## First-Run Onboarding

```mermaid
flowchart TD
  Start[App Opens] --> Check{Onboarding Complete?}
  Check -->|Yes| Main[Main App]
  Check -->|No| Profile[Collect User Name]
  Profile --> Sync[Choose Sync Setup]
  Sync --> Search[Choose Local Search Model Setup]
  Search --> Save[Save Settings]
  Save --> Mark[Set app.onboarding_completed]
  Mark --> Main
```

## Local CRUD Flow

```mermaid
sequenceDiagram
  participant U as User
  participant UI as React Feature View
  participant SDK as Generated SDK
  participant CMD as Tauri Command
  participant Repo as Repository
  participant DB as Local DB

  U->>UI: create or update journal/task/goal
  UI->>SDK: call generated hook/mutation
  SDK->>CMD: invoke Tauri command
  CMD->>Repo: validate and persist
  Repo->>DB: insert/update rows
  DB-->>Repo: result
  Repo-->>CMD: domain model
  CMD-->>SDK: response
  SDK-->>UI: update query cache
```

## Local Search Flow

```mermaid
sequenceDiagram
  participant U as User
  participant UI as Command Palette
  participant Search as Search Command
  participant Index as Search Index
  participant DB as Local DB

  U->>UI: search journal or tasks
  UI->>Search: query entry/task index
  Search->>Index: keyword, semantic, or hybrid search
  Index->>DB: read indexed journal/task documents
  DB-->>Index: matches
  Index-->>Search: ranked results
  Search-->>UI: entry/task destinations
```

## Sync Flow

```mermaid
sequenceDiagram
  participant D as Desktop App
  participant Engine as Sync Engine
  participant Server as Sync Server
  participant SDB as Sync Server DB
  participant Blob as Blob Store

  D->>Engine: configure sync
  Engine->>Server: enroll with server seed phrase
  Server->>SDB: store device token hash
  Server-->>Engine: device token and salt
  Engine->>D: store sync settings

  D->>Engine: sync now
  Engine->>Engine: encrypt pending local changes
  Engine->>Server: push encrypted batch with device auth
  Server->>SDB: store encrypted changes
  Server-->>Engine: ok
  Engine->>Server: pull encrypted changes after cursor
  Server-->>Engine: encrypted changes and next cursor
  Engine->>Engine: decrypt and apply changes

  Engine->>Server: upload/fetch encrypted image and video blobs
  Server->>Blob: read/write blobs
```

## Updater Flow

```mermaid
flowchart TD
  Settings[Settings What's New] --> Check[check_for_updates]
  AppFocus[App Focus or Startup] --> AutoCheck{Auto-check Enabled?}
  AutoCheck -->|Yes| Check
  AutoCheck -->|No| Idle[Do Nothing]
  Check --> Available{Update Available?}
  Available -->|No| Latest[Show Latest State]
  Available -->|Yes| Notify[Show Update Notification]
  Notify --> Choices{User Choice}
  Choices --> Install[Download and Install]
  Choices --> Skip[Skip Version]
  Choices --> Later[Dismiss]
  Install --> Restart[Updater Restarts App]
  Skip --> SaveSkip[Persist Skipped Version]
```

## V1 Feature Surface

```mermaid
flowchart LR
  V1[V1 App Surface] --> Journal[Journal]
  V1 --> Tasks[Tasks]
  V1 --> Goals[Goals]
  V1 --> Activity[Activity Heatmap]
  V1 --> Trash[Trash]
  V1 --> Media[Image and Video Media]
  V1 --> Search[Journal and Task Search]
  V1 --> Settings[Settings]
  V1 --> Sync[Encrypted Sync]
  V1 --> Updater[Updater]
```
