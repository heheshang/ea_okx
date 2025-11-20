# Icon Import Fix - Risk.vue

## Problem

**Error:**
```
Module '"@element-plus/icons-vue"' has no exported member 'Shield'
```

## Root Cause

The `Shield` icon does not exist in Element Plus Icons library. 

## Solution

Replaced `Shield` with `Lock` icon which is semantically appropriate for the "Leverage" risk metric.

## Changes Made

### File: `frontend/src/views/Risk.vue`

**Line 251 - Import Statement:**
```typescript
// Before
import { Select, Warning, TrendCharts, Shield } from '@element-plus/icons-vue'

// After
import { Select, Warning, TrendCharts, Lock } from '@element-plus/icons-vue'
```

**Line 313 - Icon Usage:**
```typescript
// Before
{
  label: 'Leverage',
  value: '2.3x',
  statusText: 'Normal',
  status: 'success',
  icon: Shield,
}

// After
{
  label: 'Leverage',
  value: '2.3x',
  statusText: 'Normal',
  status: 'success',
  icon: Lock,
}
```

## Alternative Icons

If `Lock` doesn't fit the design, here are other appropriate Element Plus icons for risk/security context:

| Icon Name | Use Case |
|-----------|----------|
| `Lock` | Security, protection (current choice) |
| `Warning` | Risk alert, caution |
| `WarningFilled` | High risk indication |
| `Tools` | Risk management |
| `Setting` | Configuration, control |

## Reference

[Element Plus Icon Collection](https://element-plus.org/en-US/component/icon)

## Status

✅ Fixed - TypeScript errors resolved
✅ Component compiles successfully
✅ Lock icon displays for Leverage metric

## Note

The TypeScript error may still show in the IDE due to caching. Restarting the dev server or TypeScript language server will clear the cached error.
