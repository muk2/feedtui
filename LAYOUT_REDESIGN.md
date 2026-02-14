# Resizable & Customizable Widgets - Implementation Plan

## Overview

This document outlines the implementation plan for Issue #49 - redesigning FeedTUI's layout system to support resizable, customizable widgets with drag-and-reorder capabilities.

---

## Current Architecture

### Existing Layout System

The current implementation uses a simple grid-based layout:

```rust
// Current approach in app.rs:render()
let (max_row, max_col) = self.calculate_grid_dimensions();

// Each widget has a fixed position (row, col)
let pos = widget.position();  // Returns (usize, usize)

// Widgets are rendered in equal-sized grid cells
```

**Limitations:**
- All widgets have the same size
- No dynamic resizing
- No drag-and-reorder
- Layout is determined solely by widget positions in config
- No edit mode for runtime customization

---

## Proposed Architecture

### 1. Widget Size System

#### Size Enum
```rust
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum WidgetSize {
    Small,      // 1x1 grid cells
    Medium,     // 2x1 grid cells
    Large,      // 2x2 grid cells
    FullWidth,  // Full row width
}

impl WidgetSize {
    pub fn cells(&self) -> (u16, u16) {
        match self {
            WidgetSize::Small => (1, 1),
            WidgetSize::Medium => (2, 1),
            WidgetSize::Large => (2, 2),
            WidgetSize::FullWidth => (0, 1), // 0 = full width
        }
    }
}
```

#### Updated Widget Trait
```rust
pub trait FeedWidget: Send + Sync {
    // ... existing methods ...

    // New methods for layout system
    fn size(&self) -> WidgetSize;
    fn set_size(&mut self, size: WidgetSize);
    fn min_size(&self) -> WidgetSize { WidgetSize::Small }
    fn max_size(&self) -> WidgetSize { WidgetSize::Large }
}
```

### 2. Layout Manager

```rust
pub struct LayoutManager {
    grid: Grid,
    edit_mode: bool,
    selected_widget_id: Option<String>,
}

pub struct Grid {
    rows: u16,
    cols: u16,
    cells: Vec<Vec<Option<WidgetPlacement>>>,
}

#[derive(Debug, Clone)]
pub struct WidgetPlacement {
    widget_id: String,
    position: GridPosition,
    size: WidgetSize,
    z_index: u16,
}

#[derive(Debug, Clone, Copy)]
pub struct GridPosition {
    row: u16,
    col: u16,
}

impl LayoutManager {
    pub fn new(terminal_size: (u16, u16)) -> Self;

    pub fn place_widget(&mut self, widget_id: String, pos: GridPosition, size: WidgetSize) -> Result<(), LayoutError>;

    pub fn move_widget(&mut self, widget_id: &str, new_pos: GridPosition) -> Result<(), LayoutError>;

    pub fn resize_widget(&mut self, widget_id: &str, new_size: WidgetSize) -> Result<(), LayoutError>;

    pub fn check_collision(&self, pos: GridPosition, size: WidgetSize, exclude: Option<&str>) -> bool;

    pub fn get_render_areas(&self, frame_area: Rect) -> Vec<(String, Rect)>;

    pub fn save_layout(&self, path: &Path) -> Result<()>;

    pub fn load_layout(&mut self, path: &Path) -> Result<()>;
}

#[derive(Debug, Error)]
pub enum LayoutError {
    #[error("Widget collision detected")]
    Collision,

    #[error("Position out of bounds")]
    OutOfBounds,

    #[error("Widget not found: {0}")]
    WidgetNotFound(String),
}
```

### 3. Edit Mode System

```rust
#[derive(Debug, Clone)]
pub struct EditMode {
    active: bool,
    selected_widget: Option<String>,
    ghost_position: Option<GridPosition>,  // Preview position while moving
    show_grid_overlay: bool,
}

impl EditMode {
    pub fn toggle(&mut self);
    pub fn select_widget(&mut self, widget_id: String);
    pub fn move_selection(&mut self, direction: Direction);
    pub fn cycle_size(&mut self);
}
```

### 4. Configuration Schema

```toml
# New layout configuration format
[layout]
grid_cols = 3
grid_rows = 3

[[layout.widgets]]
id = "creature-0-0"
type = "creature"
title = "Tui"
size = "medium"  # small | medium | large | fullwidth
position = { row = 0, col = 0 }
# ... widget-specific config ...

[[layout.widgets]]
id = "hackernews-0-1"
type = "hackernews"
title = "Hacker News"
size = "medium"
position = { row = 0, col = 2 }
# ... widget-specific config ...
```

### 5. Rendering System

```rust
impl App {
    fn render(&mut self, frame: &mut Frame) {
        let area = frame.area();

        if self.layout_manager.edit_mode {
            self.render_edit_mode(frame, area);
        } else {
            self.render_normal_mode(frame, area);
        }

        // Overlays (creature menu, article reader, etc.)
        self.render_overlays(frame, area);
    }

    fn render_normal_mode(&self, frame: &mut Frame, area: Rect) {
        // Get calculated render areas for each widget
        let render_areas = self.layout_manager.get_render_areas(area);

        for (widget_id, widget_area) in render_areas {
            if let Some(widget) = self.find_widget_by_id(&widget_id) {
                widget.render(frame, widget_area, widget_id == self.selected_widget_id);
            }
        }
    }

    fn render_edit_mode(&self, frame: &mut Frame, area: Rect) {
        // Render widgets with edit mode indicators
        self.render_normal_mode(frame, area);

        // Render grid overlay
        if self.edit_mode.show_grid_overlay {
            self.render_grid_overlay(frame, area);
        }

        // Render edit mode UI
        self.render_edit_ui(frame, area);
    }

    fn render_grid_overlay(&self, frame: &mut Frame, area: Rect) {
        // Draw grid lines to show cell boundaries
        // Highlight selected widget
        // Show ghost preview of widget being moved
    }

    fn render_edit_ui(&self, frame: &mut Frame, area: Rect) {
        // Show keybinding help
        // Show current widget info (size, position)
        // Show available actions
    }
}
```

---

## Implementation Phases

### Phase 1: Core Infrastructure
1. Create `LayoutManager` module
2. Implement `WidgetSize` enum
3. Extend `FeedWidget` trait with size methods
4. Update all existing widgets to support sizing
5. Implement basic collision detection

### Phase 2: Configuration System
1. Update `Config` structs to include size information
2. Implement layout serialization/deserialization
3. Add layout migration for existing configs
4. Create layout validation

### Phase 3: Rendering System
1. Refactor `App::render()` to use LayoutManager
2. Implement dynamic grid layout calculation
3. Handle different widget sizes
4. Ensure responsive resizing

### Phase 4: Edit Mode
1. Implement edit mode toggle
2. Add visual indicators (grid overlay, selection highlight)
3. Implement widget selection navigation
4. Create edit mode UI (keybinding help, widget info)

### Phase 5: Interactive Controls
1. Implement move widget (arrow keys in edit mode)
2. Implement resize widget (cycle sizes)
3. Implement collision prevention
4. Add ghost preview while moving

### Phase 6: Persistence
1. Implement layout save/load
2. Add auto-save on layout changes
3. Create layout reset functionality
4. Add layout export/import

---

## Keybindings

### Normal Mode
- `e` - Toggle edit mode

### Edit Mode
- `Esc` - Exit edit mode
- `Tab` / `Shift+Tab` - Select next/previous widget
- `←↑↓→` - Move selected widget
- `+` / `-` - Cycle widget size (small → medium → large → fullwidth → small)
- `f` - Toggle full-screen for selected widget
- `g` - Toggle grid overlay
- `Enter` - Confirm move
- `r` - Reset layout to default
- `s` - Save current layout

---

## Technical Considerations

### Collision Detection
```rust
fn check_collision(&self, pos: GridPosition, size: WidgetSize, exclude: Option<&str>) -> bool {
    let (width, height) = size.cells();

    for row in pos.row..(pos.row + height) {
        for col in pos.col..(pos.col + width) {
            if let Some(Some(placement)) = self.grid.cells.get(row).and_then(|r| r.get(col)) {
                if exclude.map_or(true, |id| id != placement.widget_id) {
                    return true;  // Collision detected
                }
            }
        }
    }

    false
}
```

### Auto-Reflow
When a widget is resized, surrounding widgets should shift to prevent overlap:

```rust
fn auto_reflow(&mut self, changed_widget_id: &str) {
    // 1. Find all widgets that collide with changed widget
    // 2. Calculate new positions for colliding widgets
    // 3. Recursively check for new collisions
    // 4. Update all affected widget positions
}
```

### Terminal Resize Handling
When the terminal is resized, the layout should adapt:

```rust
fn handle_terminal_resize(&mut self, new_size: (u16, u16)) {
    // Recalculate grid dimensions
    // Adjust widget positions if needed
    // Trigger re-render
}
```

---

## File Structure

```
src/
├── layout/
│   ├── mod.rs              # Module exports
│   ├── manager.rs          # LayoutManager implementation
│   ├── grid.rs             # Grid and cell management
│   ├── placement.rs        # WidgetPlacement logic
│   ├── collision.rs        # Collision detection
│   └── persistence.rs      # Save/load layout
├── ui/
│   ├── widgets/
│   │   ├── mod.rs          # Updated widget trait
│   │   └── ...             # Existing widgets updated
│   └── edit_mode.rs        # Edit mode rendering
└── app.rs                  # Updated to use LayoutManager
```

---

## Migration Strategy

### Backward Compatibility
Existing configs without size information should work:

```rust
impl Default for WidgetSize {
    fn default() -> Self {
        WidgetSize::Medium  // Default for backward compatibility
    }
}

// During config load:
let size = config.size.unwrap_or_default();
```

### Config Migration
```rust
pub fn migrate_config_v1_to_v2(old_config: &Path) -> Result<()> {
    // Read old config
    // Add default sizes to all widgets
    // Write new config
    // Backup old config
}
```

---

## Testing Strategy

1. **Unit Tests**
   - Collision detection
   - Size calculations
   - Position validation

2. **Integration Tests**
   - Layout save/load
   - Widget placement
   - Edit mode interactions

3. **Manual Testing**
   - Terminal resize behavior
   - Edit mode UX
   - Different widget size combinations

---

## Performance Considerations

- Cache calculated render areas
- Only recalculate layout on changes
- Efficient collision detection (spatial indexing)
- Minimal redraws in edit mode

---

## Future Enhancements

- Mouse support for drag-and-drop
- Multiple layout profiles
- Layout templates
- Visual layout editor
- Widget snapping to grid
- Custom grid sizes
- Z-index management for overlapping widgets

---

## Estimated Implementation Time

- Phase 1: 4-6 hours
- Phase 2: 2-3 hours
- Phase 3: 4-5 hours
- Phase 4: 3-4 hours
- Phase 5: 4-5 hours
- Phase 6: 2-3 hours

**Total: 19-26 hours** for complete implementation

---

## References

- Issue #49: https://github.com/muk2/feedtui/issues/49
- Ratatui Layout: https://docs.rs/ratatui/latest/ratatui/layout/
- TUI Layout Systems: Research tiling window managers (i3, sway) for inspiration
