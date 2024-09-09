'use client'
import { cn } from '@gendam/tailwind/utils'
import * as TabsPrimitive from '@radix-ui/react-tabs'
import * as React from 'react'

const TabsList = React.forwardRef<
  React.ElementRef<typeof TabsPrimitive.List>,
  React.ComponentPropsWithoutRef<typeof TabsPrimitive.List>
>(({ className, ...props }, ref) => (
  <TabsPrimitive.List
    ref={ref}
    className={cn('text-ink border-app-line inline-flex items-center justify-center border-b', className)}
    {...props}
  />
))
TabsList.displayName = TabsPrimitive.List.displayName

const TabsTrigger = React.forwardRef<
  React.ElementRef<typeof TabsPrimitive.Trigger>,
  React.ComponentPropsWithoutRef<typeof TabsPrimitive.Trigger>
>(({ className, ...props }, ref) => (
  <TabsPrimitive.Trigger
    ref={ref}
    className={cn(
      'focus-visible:ring-ring inline-flex items-center justify-center whitespace-nowrap px-3 py-1 text-sm ring-offset-transparent transition-all focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-offset-2 disabled:pointer-events-none disabled:opacity-50 data-[state=active]:font-bold data-[state=active]:shadow-[0_2px_0_0_rgba(0,0,0,0)] data-[state=active]:shadow-current',
      className,
    )}
    {...props}
  />
))
TabsTrigger.displayName = TabsPrimitive.Trigger.displayName

const TabsContent = React.forwardRef<
  React.ElementRef<typeof TabsPrimitive.Content>,
  React.ComponentPropsWithoutRef<typeof TabsPrimitive.Content>
>(({ className, ...props }, ref) => (
  <TabsPrimitive.Content
    ref={ref}
    className={cn(
      'focus-visible:ring-ring mt-2 ring-offset-transparent focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-offset-2',
      className,
    )}
    {...props}
  />
))
TabsContent.displayName = TabsPrimitive.Content.displayName

const Tabs = {
  Root: TabsPrimitive.Root,
  Content: TabsContent,
  List: TabsList,
  Trigger: TabsTrigger,
}

export { Tabs, TabsPrimitive }
