# TagBox Freya GUI - Next Steps Development Plan

## Phase 1: Complete Core Functionality (1 week)

### 1.1 Edit Page Operations
- [ ] Wire up the Save button to actually update file metadata using TagBoxService
- [ ] Implement the Delete functionality with confirmation dialog
- [ ] Add Reset to Original functionality
- [ ] Implement Re-extract Metadata feature
- [ ] Add success/error notifications after operations

### 1.2 Semantic Links View
- [ ] Create the fourth page as specified in the design guide
- [ ] Implement link creation between files
- [ ] Show bidirectional links visualization
- [ ] Add link removal functionality
- [ ] Create link type selection (related, references, etc.)

### 1.3 Functional Category Tree
- [ ] Fetch actual categories from database on startup
- [ ] Implement category click to filter files
- [ ] Show file count per category
- [ ] Add expand/collapse for nested categories
- [ ] Implement category creation/editing

## Phase 2: Essential UX Improvements (1 week)

### 2.1 Keyboard Shortcuts
- [ ] Ctrl/Cmd + F: Focus search
- [ ] Ctrl/Cmd + I: Open import page
- [ ] Ctrl/Cmd + E: Edit selected file
- [ ] Delete key: Delete selected file
- [ ] Arrow keys: Navigate file list
- [ ] Enter: Open file preview

### 2.2 Context Menus
- [ ] Right-click on file: Edit, Delete, Link, Export
- [ ] Right-click on category: Filter, Edit, Delete
- [ ] Right-click on empty space: Import, Refresh

### 2.3 File Preview Enhancements
- [ ] Show actual file preview for PDFs (first page thumbnail)
- [ ] Display text content for text files
- [ ] Show metadata in a formatted view
- [ ] Add "Open in System" button
- [ ] Implement copy file path functionality

### 2.4 Visual Feedback
- [ ] Hover effects on clickable elements
- [ ] Selection highlight in file list
- [ ] Loading spinners for async operations
- [ ] Success/error toast notifications
- [ ] Progress bar for import operations

## Phase 3: Performance Optimizations (3 days)

### 3.1 Virtual Scrolling
- [ ] Implement virtual list for file display
- [ ] Only render visible items + buffer
- [ ] Smooth scrolling experience
- [ ] Maintain selection state during scroll

### 3.2 Search Optimizations
- [ ] Add debouncing to search input (300ms)
- [ ] Implement search result pagination
- [ ] Add "Load More" or infinite scroll
- [ ] Cache recent search results

### 3.3 Data Caching
- [ ] Cache categories on startup
- [ ] Cache author list for autocomplete
- [ ] Implement incremental updates
- [ ] Add refresh mechanisms

## Phase 4: Testing & Quality (3 days)

### 4.1 Component Tests
```rust
// Example test structure
#[cfg(test)]
mod tests {
    use freya_testing::*;
    
    #[tokio::test]
    async fn test_file_card_selection() {
        let mut app = launch_test(FileCard);
        app.click(".file-card");
        assert!(app.query(".selected").exists());
    }
}
```

### 4.2 Integration Tests
- [ ] Test complete workflows (search → select → edit)
- [ ] Test import with various file types
- [ ] Test error handling scenarios
- [ ] Test state persistence

### 4.3 Performance Benchmarks
- [ ] Measure render time for 1000+ files
- [ ] Test memory usage with large datasets
- [ ] Profile search performance
- [ ] Optimize hot paths

## Phase 5: Documentation & Deployment (3 days)

### 5.1 User Documentation
- [ ] Create user guide with screenshots
- [ ] Document keyboard shortcuts
- [ ] Explain DSL search syntax
- [ ] Create video tutorials

### 5.2 Developer Documentation
- [ ] Document component architecture
- [ ] Explain state management approach
- [ ] Create contribution guidelines
- [ ] Document testing approach

### 5.3 Release Pipeline
- [ ] Set up GitHub Actions for CI/CD
- [ ] Create release builds for all platforms
- [ ] Implement auto-update mechanism
- [ ] Create installers/packages

## Phase 6: Advanced Features (Future)

### 6.1 Batch Operations
- [ ] Multi-select with Ctrl/Cmd click
- [ ] Select all with Ctrl/Cmd + A
- [ ] Bulk edit metadata
- [ ] Bulk export/delete

### 6.2 Export Features
- [ ] Export search results to CSV
- [ ] Export file list with metadata
- [ ] Create reading lists
- [ ] Generate reports

### 6.3 Customization
- [ ] Theme selection (light/dark)
- [ ] Customizable layout
- [ ] Configurable columns in file list
- [ ] User preferences persistence

## Implementation Priority

1. **Week 1**: Complete Phase 1 (Core Functionality)
2. **Week 2**: Complete Phase 2 (UX) + Phase 3 (Performance)
3. **Week 3**: Complete Phase 4 (Testing) + Phase 5 (Documentation)
4. **Future**: Phase 6 (Advanced Features)

## Technical Considerations

### State Management
- Consider implementing undo/redo functionality
- Add optimistic updates for better UX
- Implement proper error recovery

### Architecture
- Keep service layer separate from UI
- Maintain testability with dependency injection
- Consider plugin architecture for extensibility

### Performance
- Lazy load heavy components
- Implement proper memoization
- Use web workers for heavy computations

## Success Metrics

- [ ] All 4 pages from design guide functional
- [ ] < 100ms search response time
- [ ] < 50MB memory usage for 10k files
- [ ] 80%+ test coverage
- [ ] Zero accessibility violations
- [ ] Native installers for all platforms

## Next Immediate Tasks

1. **Start with Edit Page operations** - This completes the CRUD functionality
2. **Implement Category Tree** - Essential for navigation
3. **Add keyboard shortcuts** - Improves productivity significantly
4. **Create component tests** - Ensures stability as we add features