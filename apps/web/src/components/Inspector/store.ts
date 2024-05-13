'use client'
import { create } from 'zustand'

interface InspectorState {
  show: boolean
  setShow: (show: boolean) => void
}

export const useInspector = create<InspectorState>((set) => ({
  show: false,
  setShow: (show) => set({ show }),
}))
