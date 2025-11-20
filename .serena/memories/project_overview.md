# EA OKX Project Overview

## Project Purpose
EA OKX is a quantitative trading platform with integrated algorithm management, backtesting capabilities, and risk management. It's built as a hybrid web/desktop application using Vue 3 + Tauri for cross-platform deployment.

## Tech Stack
- **Frontend**: Vue 3 (Composition API), TypeScript, Element Plus UI
- **Desktop**: Tauri 2.0 for native desktop app capabilities
- **Styling**: SCSS with design system, CSS custom properties for theming
- **State Management**: Pinia with persistence
- **Charts**: ECharts with Vue integration
- **Build**: Vite for fast development and building
- **Testing**: Vitest for unit tests, Playwright for E2E testing

## Architecture Patterns
- **Responsive Design**: Mobile-first approach with breakpoints at 768px, 1024px, 1440px
- **Component Architecture**: Layout-based with MainLayout.vue + responsive components
- **Theme System**: CSS custom properties with dark/light mode support
- **Navigation**: Sidebar for desktop, bottom navigation for mobile
- **Internationalization**: Vue-i18n for multi-language support

## Key Features
1. Dashboard with system metrics and performance charts
2. Strategy management and configuration
3. Backtesting engine with historical data analysis
4. Live trading interface with order management
5. Risk management and monitoring tools
6. Settings and configuration management

## Code Conventions
- **Naming**: camelCase for variables, PascalCase for components
- **File Structure**: Feature-based organization with clear separation of concerns
- **Styling**: SCSS with CSS custom properties, mobile-first responsive design
- **TypeScript**: Strict typing throughout the codebase
- **Vue**: Composition API with `<script setup>` syntax
- **Linting**: Follows Vue 3 and TypeScript best practices

## Development Commands
- `pnpm dev` - Start development server
- `pnpm tauri:dev` - Start Tauri development mode
- `pnpm build` - Build for web
- `pnpm tauri:build` - Build desktop app
- `pnpm test` - Run unit tests
- `pnpm test:e2e` - Run E2E tests with Playwright

## Responsive Design Implementation
- Mobile-first CSS with SCSS mixins
- Breakpoint system: mobile (<768px), tablet (768px-1024px), desktop (1024px-1440px), desktop-lg (>1440px)
- Responsive navigation: sidebar (desktop) vs bottom nav (mobile)
- Adaptive layouts using Element Plus grid system
- Touch-friendly UI with minimum 44px touch targets
- Safe area insets for mobile devices