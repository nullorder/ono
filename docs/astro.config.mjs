import { defineConfig } from 'astro/config';
import starlight from '@astrojs/starlight';
import starlightThemeRapide from 'starlight-theme-rapide';

export default defineConfig({
  site: 'https://ono.nullorder.dev',
  integrations: [
    starlight({
      plugins: [starlightThemeRapide()],
      title: 'Ono',
      description: 'Beautiful terminal UI components for Ratatui.',
      social: [
        { icon: 'github', label: 'GitHub', href: 'https://github.com/nullorder/ono' },
      ],
      editLink: {
        baseUrl: 'https://github.com/nullorder/ono/edit/main/docs/',
      },
      sidebar: [
        { label: 'Getting started', slug: 'getting-started' },
        { label: 'Components', slug: 'components' },
        { label: 'Theming', slug: 'theming' },
        { label: 'Ejecting', slug: 'eject' },
      ],
    }),
  ],
});
