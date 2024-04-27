'use client'
import * as DropdownMenuPrimitive from '@radix-ui/react-dropdown-menu'
import * as React from 'react'
import { cn } from '@gendam/tailwind/utils'

const Content = React.forwardRef<
  React.ElementRef<typeof DropdownMenuPrimitive.Content>,
  React.ComponentPropsWithoutRef<typeof DropdownMenuPrimitive.Content>
>(({ className, ...props }, ref) => (
  <DropdownMenuPrimitive.Content
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
Content.displayName = DropdownMenuPrimitive.Content.displayName

const Item = React.forwardRef<
  React.ElementRef<typeof DropdownMenuPrimitive.Item>,
  React.ComponentPropsWithoutRef<typeof DropdownMenuPrimitive.Item> & {
    variant?: 'accent' | 'destructive'
  }
>(({ className, variant='accent', ...props }, ref) => (
  <DropdownMenuPrimitive.Item
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
Item.displayName = DropdownMenuPrimitive.Item.displayName

const Label = React.forwardRef<
  React.ElementRef<typeof DropdownMenuPrimitive.Label>,
  React.ComponentPropsWithoutRef<typeof DropdownMenuPrimitive.Label>
>(({ className, ...props }, ref) => (
  <DropdownMenuPrimitive.Label
    ref={ref}
    className={cn('text-ink/50 py-1', className)}
    {...props}
  />
))
Label.displayName = DropdownMenuPrimitive.Label.displayName

const Separator = React.forwardRef<
  React.ElementRef<typeof DropdownMenuPrimitive.Separator>,
  React.ComponentPropsWithoutRef<typeof DropdownMenuPrimitive.Separator>
>(({ className, ...props }, ref) => (
  <DropdownMenuPrimitive.Separator
    ref={ref}
    className={cn('mx-1 my-1 h-px bg-app-line', className)}
    {...props}
  />
))
Separator.displayName = DropdownMenuPrimitive.Separator.displayName

const DropdownMenu = {
  Root: DropdownMenuPrimitive.Root,
  Trigger: DropdownMenuPrimitive.Trigger,
  Portal: DropdownMenuPrimitive.Portal,
  Content,
  Item,
  Label,
  Separator,
}

export {
  DropdownMenuPrimitive,
  DropdownMenu
}
