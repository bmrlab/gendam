'use client'
import { cn } from '@gendam/tailwind/utils'
import * as TooltipPrimitive from '@radix-ui/react-tooltip'
import * as React from 'react'

const Content = React.forwardRef<
  React.ElementRef<typeof TooltipPrimitive.Content>,
  React.ComponentPropsWithoutRef<typeof TooltipPrimitive.Content>
>(({ className, sideOffset = 4, ...props }, ref) => (
  <TooltipPrimitive.Content
    ref={ref}
    sideOffset={sideOffset}
    className={cn(
      'z-50 bg-app-overlay text-ink overflow-hidden rounded-md px-3 py-1.5 text-xs',
      'animate-in data-[state=closed]:animate-out',
      'fade-in-0 data-[state=closed]:fade-out-0',
      'zoom-in-95 data-[state=closed]:zoom-out-95',
      'data-[side=top]:slide-in-from-bottom-2',
      'data-[side=bottom]:slide-in-from-top-2',
      'data-[side=left]:slide-in-from-right-2',
      'data-[side=right]:slide-in-from-left-2',
      className,
    )}
    {...props}
  />
))
Content.displayName = TooltipPrimitive.Content.displayName


const Tooltip = {
  Provider: TooltipPrimitive.Provider,
  Root: TooltipPrimitive.Root,
  Trigger: TooltipPrimitive.Trigger,
  Portal: TooltipPrimitive.Portal,
  Content,
}

export {
  TooltipPrimitive,
  Tooltip
}
