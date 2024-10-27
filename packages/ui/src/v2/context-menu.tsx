'use client'
import { cn } from '@gendam/tailwind/utils'
import * as ContextMenuPrimitive from '@radix-ui/react-context-menu'
import { ChevronRightIcon } from '@radix-ui/react-icons'
import * as React from 'react'

const Content = React.forwardRef<
  React.ElementRef<typeof ContextMenuPrimitive.Content>,
  React.ComponentPropsWithoutRef<typeof ContextMenuPrimitive.Content>
>(({ className, ...props }, ref) => (
  <ContextMenuPrimitive.Content
    ref={ref}
    className={cn(
      'text-ink bg-app-box border-app-line min-w-48 rounded-md border p-1 shadow-lg',
      'data-[state=open]:animate-in data-[state=closed]:animate-out',
      'data-[state=open]:fade-in-0 data-[state=closed]:fade-out-0',
      'data-[state=open]:zoom-in-95 data-[state=closed]:zoom-out-95',
      className,
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
>(({ className, variant = 'accent', ...props }, ref) => (
  <ContextMenuPrimitive.Item
    ref={ref}
    className={cn(
      'cursor-default select-none outline-none',
      'data-[disabled]:pointer-events-none data-[disabled]:opacity-50',
      'flex items-center justify-start gap-1.5 rounded-md px-2.5 py-1.5 text-xs',
      variant === 'accent' && 'text-ink focus:bg-accent hover:bg-accent hover:text-white focus:text-white',
      variant === 'destructive' &&
        'text-red-600 hover:bg-red-500/90 hover:text-white focus:bg-red-500/90 focus:text-white',
      className,
    )}
    {...props}
  />
))
Item.displayName = ContextMenuPrimitive.Item.displayName

const Separator = React.forwardRef<
  React.ElementRef<typeof ContextMenuPrimitive.Separator>,
  React.ComponentPropsWithoutRef<typeof ContextMenuPrimitive.Separator>
>(({ className, ...props }, ref) => (
  <ContextMenuPrimitive.Separator ref={ref} className={cn('bg-app-line mx-1 my-1 h-px', className)} {...props} />
))
Separator.displayName = ContextMenuPrimitive.Separator.displayName

const SubTrigger = React.forwardRef<
  React.ElementRef<typeof ContextMenuPrimitive.SubTrigger>,
  React.ComponentPropsWithoutRef<typeof ContextMenuPrimitive.SubTrigger> & {
    inset?: boolean
  }
>(({ className, inset, children, disabled, ...props }, ref) => (
  <ContextMenuPrimitive.SubTrigger
    ref={ref}
    className={cn(
      'focus:bg-accent data-[state=open]:bg-accent flex cursor-default select-none items-center rounded-sm px-2 py-1.5 text-sm outline-none focus:text-white data-[state=open]:text-white',
      inset && 'pl-8',
      'cursor-default select-none outline-none',
      'flex items-center justify-start gap-1.5 rounded-md px-2.5 py-1.5 text-xs',
      'text-ink focus:bg-accent hover:bg-accent focus:text-white data-[state=open]:text-white',
      disabled && 'pointer-events-none opacity-50',
      className,
    )}
    {...props}
  >
    {children}
    <ChevronRightIcon className="ml-auto h-4 w-4" />
  </ContextMenuPrimitive.SubTrigger>
))
SubTrigger.displayName = ContextMenuPrimitive.SubTrigger.displayName

const SubContent = React.forwardRef<
  React.ElementRef<typeof ContextMenuPrimitive.SubContent>,
  React.ComponentPropsWithoutRef<typeof ContextMenuPrimitive.SubContent>
>(({ className, ...props }, ref) => (
  <ContextMenuPrimitive.SubContent
    ref={ref}
    className={cn(
      'bg-app-box text-ink min-w-36 overflow-hidden rounded-md border p-1 shadow-lg',
      'data-[state=open]:animate-in data-[state=closed]:animate-out',
      'data-[state=open]:fade-in-0 data-[state=closed]:fade-out-0',
      'data-[state=open]:zoom-in-95 data-[state=closed]:zoom-out-95',
      // 'data-[side=bottom]:slide-in-from-top-2 data-[side=top]:slide-in-from-bottom-2',
      // 'data-[side=right]:slide-in-from-left-2 data-[side=left]:slide-in-from-right-2',
      className,
    )}
    {...props}
  />
))
SubContent.displayName = ContextMenuPrimitive.SubContent.displayName

const ContextMenu = {
  Root: ContextMenuPrimitive.Root,
  Trigger: ContextMenuPrimitive.Trigger,
  Portal: ContextMenuPrimitive.Portal,
  Content,
  Item,
  Separator,
  Sub: ContextMenuPrimitive.Sub,
  SubTrigger,
  SubContent,
}

export { ContextMenu, ContextMenuPrimitive }
