'use client'
import { cn } from '@gendam/tailwind/utils'
import * as PopoverPrimitive from '@radix-ui/react-popover'
import * as React from 'react'

const Content = React.forwardRef<
  React.ElementRef<typeof PopoverPrimitive.Content>,
  React.ComponentPropsWithoutRef<typeof PopoverPrimitive.Content>
>(({ className, ...props }, ref) => (
  <PopoverPrimitive.Content
    ref={ref}
    className={cn(
      'bg-app-box text-ink border-app-line min-w-48 overflow-hidden rounded-md border shadow-lg outline-none',
      'data-[state=open]:animate-in data-[state=closed]:animate-out',
      'data-[state=open]:fade-in-0 data-[state=closed]:fade-out-0',
      'data-[state=open]:zoom-in-95 data-[state=closed]:zoom-out-95',
      className,
    )}
    {...props}
  />
))
Content.displayName = PopoverPrimitive.Content.displayName

const Popover = {
  Root: PopoverPrimitive.Root,
  Trigger: PopoverPrimitive.Trigger,
  Portal: PopoverPrimitive.Portal,
  Content,
}

export { Popover, PopoverPrimitive }
