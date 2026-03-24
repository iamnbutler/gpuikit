# gpuikit Component Status

## Implemented

- Avatar
- Badge
- Breadcrumb
- Button
- Card
- Checkbox
- Dropdown Menu
- Empty
- Icon Button
- Input
- Kbd
- Loading Indicator
- Progress
- Radio Group
- Separator
- Slider
- Toggle
- Tooltip
- Typography

## Not Implemented

### Easy

- Label
- Skeleton

### Medium

- Alert
- Aspect Ratio
- Button Group
- Switch
- Collapsible
- Accordion
- Toggle Group
- Tabs
- Field
- Input Group

### Out of Scope (Complex)

See [Issue #59](https://github.com/iamnbutler/gpuikit/issues/59) for full details on these components.

These components require significant infrastructure and are deferred:

#### Overlay/Positioning Components
- Popover - requires overlay positioning system
- Hover Card - requires overlay positioning system
- Dialog - requires modal overlay system
- Alert Dialog - requires modal overlay system
- Sheet - requires overlay slide-in animations
- Drawer - requires overlay slide-in animations
- Context Menu - requires overlay positioning + right-click handling
- Menubar - requires complex menu navigation
- Navigation Menu - requires complex nested menu system

#### Advanced Input Components
- Textarea - requires multi-line text editing primitives
- Command - requires command palette infrastructure
- Combobox - requires overlay + filtering logic
- Select - requires overlay dropdown
- Native Select - requires native OS integration

#### Data Display Components
- Table - requires table layout primitives
- Data Table - requires Table + sorting/filtering
- Pagination - useful alongside Table/Data Table

#### Layout Components
- Scroll Area - requires custom scroll handling
- Resizable - requires drag/resize handling
- Sidebar - requires layout management

#### Notification Components
- Toast - requires notification queue system
- Sonner - requires toast infrastructure

#### Date/Time Components
- Calendar - requires date handling + grid layout
- Date Picker - requires Calendar + overlay

#### Other Complex Components
- Carousel - requires slide animations
- Chart - requires data visualization primitives
- Form - requires form state management
- Input OTP - requires specialized input handling
- Item - unclear requirements
