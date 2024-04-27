'use client'
import { cn } from '@gendam/tailwind/utils'
import * as FormPrimitive from '@radix-ui/react-form'
import { Primitive, type ComponentPropsWithoutRef } from '@radix-ui/react-primitive'
import { cva, type VariantProps } from 'class-variance-authority'
import * as React from 'react'

const inputVariants = cva(
  'bg-app-overlay text-ink border border-app-line outline-none',
  {
    variants: {
      size: {
        xs: 'h-6 px-2 text-xs font-medium rounded',
        sm: 'h-7 px-3 py-1 text-xs font-medium rounded-md',
        md: 'h-8 px-3 py-1 text-sm font-medium rounded-md',
        lg: 'h-9 px-3 py-1 text-md font-medium rounded-md',
      },
    },
    defaultVariants: {
      size: 'md',
    },
  }
)

const Input = React.forwardRef<
  React.ElementRef<typeof Primitive.input>,
  Omit<ComponentPropsWithoutRef<typeof Primitive.input>, 'size'> &
    VariantProps<typeof inputVariants>
>(({ className, size, ...props }, ref) => (
  <FormPrimitive.Control asChild>
    <input ref={ref} className={cn(inputVariants({ size }), className)} {...props}></input>
  </FormPrimitive.Control>
))
Input.displayName = "Form.Input"

const Form = {
  Root: FormPrimitive.Root,
  Field: FormPrimitive.Field,
  Label: FormPrimitive.Label,
  Control: FormPrimitive.Control,
  Input,
}

export {
  FormPrimitive,
  Form,
}
