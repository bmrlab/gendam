'use client'
import { create } from 'zustand'

interface InspectorState {
  show: boolean
  setShow: (show: boolean) => void
  viewerHeight: number
  setViewerHeight: (viewerHeight: number) => void
}

export const useInspector = create<InspectorState>((set) => ({
  show: false,
  setShow: (show) => set({ show }),
  viewerHeight: 192,
  setViewerHeight: (viewerHeight) => set({ viewerHeight }),
}))
