export default defineAppConfig({
  ui: {
    colors: { primary: 'stone', neutral: 'stone' },
    card: { variants: { variant: { outline: { root: 'bg-elevated' } } } },
    modal: { slots: { content: 'bg-elevated' } },
    input: { variants: { variant: { outline: 'ring-(--input)' } } }
  }
})
