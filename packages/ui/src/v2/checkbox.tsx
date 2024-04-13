'use client'
import { cn } from '@muse/tailwind/utils'
import * as CheckboxPrimitive from '@radix-ui/react-checkbox'
import * as React from 'react'
import Icon from '../icons'

const Root = React.forwardRef<
  React.ElementRef<typeof CheckboxPrimitive.Root>,
  React.ComponentPropsWithoutRef<typeof CheckboxPrimitive.Root>
>(({ className, ...props }, ref) => (
  <CheckboxPrimitive.Root
    ref={ref}
    className={cn(
      'h-4 w-4 appearance-none outline-none cursor-default',
      'flex items-center justify-center rounded',
      'bg-app text-ink border border-ink/30',
      'data-[state="checked"]:bg-accent data-[state="checked"]:text-white data-[state="checked"]:border-transparent',
      className,
    )}
    {...props}
  />
))
Root.displayName = CheckboxPrimitive.Root.displayName

const Indicator = React.forwardRef<
  React.ElementRef<typeof CheckboxPrimitive.Indicator>,
  React.ComponentPropsWithoutRef<typeof CheckboxPrimitive.Indicator>
>(({ className, ...props }, ref) => (
  <CheckboxPrimitive.Indicator ref={ref} className={cn('p-px', className)} {...props}>
    <Icon.Check className="w-full h-full" />
  </CheckboxPrimitive.Indicator>
))
Indicator.displayName = CheckboxPrimitive.Indicator.displayName

const Checkbox = {
  Root,
  Indicator,
}

export { Checkbox, CheckboxPrimitive }
