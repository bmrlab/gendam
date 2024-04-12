'use client'
import { Form } from '@muse/ui/v2/form'

export default function Forms() {
  return (
    <div className="flex flex-wrap gap-4">
      <Form.Root className="w-64">
        {['xs', 'sm', 'md', 'lg'].map((size) => (
          <Form.Field key={size} name={size} className="my-2">
            <Form.Input
              size={size as any}
              placeholder={size}
              className="w-full block"
            ></Form.Input>
          </Form.Field>
        ))}
      </Form.Root>
    </div>
  )
}
