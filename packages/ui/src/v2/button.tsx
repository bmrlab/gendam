import { Slot } from '@radix-ui/react-slot'
import { cva, type VariantProps } from 'class-variance-authority'
import * as React from 'react'

import { cn } from '../utils'

const buttonVariants = cva(
  'inline-flex items-center justify-center whitespace-nowrap transition-colors focus-visible:outline-none cursor-default border disabled:opacity-50 disabled:pointer-events-none',
  {
    variants: {
      variant: {
        ghost: 'text-ink bg-transparent hover:bg-app-hover border-transparent shadow-transparent',
        outline: 'text-ink bg-transparent hover:bg-app-hover border-app-line',
        accent: 'text-white bg-accent hover:bg-accent/90 border-accent/90',
        destructive: 'text-white bg-red-400 hover:bg-red-400/90 border-red-400/90',
      },
      size: {
        xs: 'h-6 px-2 text-xs font-medium rounded shadow-sm',
        sm: 'h-7 px-3 py-1 text-xs font-medium rounded-md shadow-sm',
        md: 'h-8 px-4 py-1 text-sm font-medium rounded-md shadow-sm',
        lg: 'h-9 px-8 py-1 text-md font-medium rounded-md shadow-sm',
      }
    },
    defaultVariants: {
      variant: 'outline',
      size: 'md',
    },
  },
)

export interface ButtonProps extends React.ButtonHTMLAttributes<HTMLButtonElement>, VariantProps<typeof buttonVariants> {
  asChild?: boolean
}

const Button = React.forwardRef<HTMLButtonElement, ButtonProps>(
  ({ className, variant, size, asChild = false, ...props }, ref) => {
    const Comp = asChild ? Slot : 'button'
    return <Comp className={cn(buttonVariants({ variant, size, className }))} ref={ref} {...props} />
  },
)
Button.displayName = 'Button'

export { Button, buttonVariants }
