'use client'
import * as React from 'react'
import * as ContextMenuPrimitive from '@radix-ui/react-context-menu'

import { cn, twx } from '@gendam/tailwind/utils'
import { Trigger } from '@radix-ui/react-dialog'

const Content = React.forwardRef<
  React.ElementRef<typeof ContextMenuPrimitive.Content>,
  React.ComponentPropsWithoutRef<typeof ContextMenuPrimitive.Content>
>(({ className, ...props }, ref) => (
  <ContextMenuPrimitive.Content
    ref={ref}
    className={cn(
      'min-w-48 rounded-md text-ink bg-app-box border border-app-line p-1 shadow-lg',
      'data-[state=open]:animate-in data-[state=closed]:animate-out',
      'data-[state=open]:fade-in-0 data-[state=closed]:fade-out-0',
      'data-[state=open]:zoom-in-95 data-[state=closed]:zoom-out-95',
      className
    )}
    {...props}
  />
))
Content.displayName = ContextMenuPrimitive.Content.displayName

const Item = React.forwardRef<
  React.ElementRef<typeof ContextMenuPrimitive.Item>,
  React.ComponentPropsWithoutRef<typeof ContextMenuPrimitive.Item> & {
    variant?: 'accent' | 'destructive'
  }
>(({ className, variant='accent', ...props }, ref) => (
  <ContextMenuPrimitive.Item
    ref={ref}
    className={cn(
      'cursor-default select-none outline-none',
      'data-[disabled]:pointer-events-none data-[disabled]:opacity-50',
      'flex items-center justify-start gap-1.5 rounded-md px-2.5 py-1.5 text-xs',
      variant === 'accent' && 'text-ink focus:bg-accent focus:text-white hover:bg-accent hover:text-white',
      variant === 'destructive' && 'text-red-600 focus:bg-red-500/90 focus:text-white hover:bg-red-500/90 hover:text-white',
      className
    )}
    {...props}
  />
))
Item.displayName = ContextMenuPrimitive.Item.displayName

const Separator = React.forwardRef<
  React.ElementRef<typeof ContextMenuPrimitive.Separator>,
  React.ComponentPropsWithoutRef<typeof ContextMenuPrimitive.Separator>
>(({ className, ...props }, ref) => (
  <ContextMenuPrimitive.Separator
    ref={ref}
    className={cn('mx-1 my-1 h-px bg-app-line', className)}
    {...props}
  />
))
Separator.displayName = ContextMenuPrimitive.Separator.displayName

const ContextMenu = {
  Root: ContextMenuPrimitive.Root,
  Trigger: ContextMenuPrimitive.Trigger,
  Portal: ContextMenuPrimitive.Portal,
  Content,
  Item,
  Separator,
}

export {
  ContextMenuPrimitive,
  ContextMenu
}
