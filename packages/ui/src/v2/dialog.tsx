'use client'
import * as DialogPrimitive from '@radix-ui/react-dialog'
import * as React from 'react'
import { cn } from '@muse/tailwind/utils'

const Overlay = React.forwardRef<
  React.ElementRef<typeof DialogPrimitive.Overlay>,
  React.ComponentPropsWithoutRef<typeof DialogPrimitive.Overlay>
>(({ className, children, ...props }, ref) => (
  <DialogPrimitive.Overlay
    ref={ref}
    className={cn(
      'fixed inset-0 z-50 bg-black/30',
      'data-[state=open]:animate-in data-[state=closed]:animate-out',
      'data-[state=open]:fade-in-0 data-[state=closed]:fade-out-0',
      className,
    )}
    {...props}
  >
    {children}
  </DialogPrimitive.Overlay>
))
Overlay.displayName = DialogPrimitive.Overlay.displayName

const Content = React.forwardRef<
  React.ElementRef<typeof DialogPrimitive.Content>,
  React.ComponentPropsWithoutRef<typeof DialogPrimitive.Content>
>(({ className, children, ...props }, ref) => (
  <DialogPrimitive.Content
    ref={ref}
    className={cn(
      'fixed z-50 left-[50%] top-[47%] translate-x-[-50%] translate-y-[-50%] overflow-auto',
      'min-w-[20rem] max-w-[90%] min-h-[10rem] max-h-[90%]',
      'rounded-lg border border-app-line bg-app-box text-ink shadow-lg',
      'data-[state=open]:animate-in data-[state=closed]:animate-out',
      'data-[state=open]:fade-in-0 data-[state=closed]:fade-out-0',
      // 'data-[state=open]:zoom-in-95 data-[state=closed]:zoom-out-95',
      'data-[state=open]:slide-in-from-left-1/2 data-[state=closed]:slide-out-to-left-1/2 ',
      'data-[state=open]:slide-in-from-top-[60%] data-[state=closed]:slide-out-to-top-[60%]',
      'data-[state=open]:duration-300 data-[state=close]:duration-300',  // 如果不加 data[], duration-x 的优先级不够
      className,
    )}
    {...props}
  >
    {children}
  </DialogPrimitive.Content>
))
Content.displayName = DialogPrimitive.Content.displayName

const Dialog = {
  Root: DialogPrimitive.Root,
  Portal: DialogPrimitive.Portal,
  Close: DialogPrimitive.Close,
  Overlay,
  Content,
}

export {
  DialogPrimitive,
  Dialog,
}
