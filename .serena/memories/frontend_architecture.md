# Frontend Architecture Analysis

## Application Structure
```
src/
├── components/          # Reusable UI components
│   ├── ResponsiveHeader.vue
│   └── MobileBottomNav.vue
├── layouts/            # Layout components
│   └── MainLayout.vue  # Main app layout with sidebar
├── views/              # Page components
│   ├── Dashboard.vue
│   ├── Strategies.vue
│   ├── Backtest.vue
│   ├── Trading.vue
│   ├── Risk.vue
│   └── Settings.vue
├── composables/        # Vue composables
│   ├── useResponsive.ts
│   ├── useTheme.ts
│   └── useTauriEvents.ts
├── stores/             # Pinia state management
├── router/             # Vue Router configuration
└── styles/             # SCSS styling system
    ├── variables.scss  # Design tokens
    ├── utilities.scss  # Utility classes and mixins
    ├── animations.scss
    ├── buttons.scss
    └── theme.scss
```

## Responsive Design Strategy

### Breakpoint System
- **Mobile**: < 768px
- **Tablet**: 768px - 1024px  
- **Desktop**: 1024px - 1440px
- **Desktop Large**: > 1440px

### Navigation Architecture
1. **Desktop**: Sidebar navigation + header
   - Fixed sidebar (200px width)
   - Horizontal header with breadcrumbs
   - Full menu visibility

2. **Mobile**: Bottom tab navigation + header
   - Bottom navigation (5 primary tabs)
   - Responsive header with essential actions
   - Collapsible search functionality

### Layout Components
- **MainLayout.vue**: Orchestrates responsive layout
- **ResponsiveHeader.vue**: Adaptive header component
- **MobileBottomNav.vue**: Mobile-specific navigation

### Responsive Utilities
- SCSS mixins for breakpoint queries
- useResponsive composable for reactive breakpoint detection
- Utility classes for responsive behavior
- Element Plus grid system integration

## Design System

### Color Scheme
- **Dark Mode**: GitHub-style dark theme
- **Light Mode**: Clean, professional appearance
- **Trading Colors**: Green (long/buy), Red (short/sell)
- **Semantic Colors**: Success, warning, danger, info states

### Typography
- Roboto font family with system fallbacks
- Responsive font scaling (14px base on mobile, 16px on desktop)
- Consistent line heights and font weights

### Spacing System
- 4px base unit for consistent spacing
- Mobile-optimized padding and margins
- Touch-friendly target sizes (minimum 44px)

### Interactive Elements
- Smooth transitions (150ms-350ms)
- Hover and active states
- Loading animations
- Focus indicators for accessibility

## Mobile Optimization Features
1. **Touch-Friendly Interface**: Minimum 44px touch targets
2. **Safe Area Support**: Handles notches and home indicators
3. **Adaptive Layouts**: Reorganized content for smaller screens
4. **Performance**: Optimized assets and reduced animations
5. **Gesture Support**: Swipe navigation and touch interactions

## Component Patterns
- **Composition API**: Modern Vue 3 patterns with `<script setup>`
- **TypeScript**: Full type safety throughout
- **Props/Emit Patterns**: Clear component communication
- **Reactive Data**: Computed properties and watchers
- **Lifecycle Management**: Proper cleanup and initialization