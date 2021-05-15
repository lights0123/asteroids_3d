import pkg from 'mdsvex';
import sveltePreprocess from 'svelte-preprocess';
import staticAdapter from '@sveltejs/adapter-static';
import svelteSVG from 'rollup-plugin-svelte-svg';
import { promises as fs } from 'fs';
import { createRequire } from 'module';
const { mdsvex } = pkg;

const require = createRequire(import.meta.url);
/** @type {import('@sveltejs/kit').Config} */
export default {
    // Consult https://github.com/sveltejs/svelte-preprocess
    // for more information about preprocessors
    preprocess: [
        sveltePreprocess({
            defaults: {
                style: 'postcss',
            },
            postcss: true,
        }),
        mdsvex({
            extensions: ['.md'],
            smartypants: {
                dashes: 'oldschool',
            },
            remarkPlugins: [],
            rehypePlugins: [
                require('rehype-slug'),
                [
                    require('rehype-autolink-headings'),
                    {
                        behavior: 'wrap',
                    },
                ],
            ],
        }),
    ],
    extensions: ['.svelte', '.md'],

    kit: {
        // By default, `npm run build` will create a standard Node app.
        // You can create optimized builds for different platforms by
        // specifying a different adapter
        adapter: staticAdapter(),

        // hydrate the <div id="svelte"> element in src/app.html
        target: '#svelte',

        paths: {
            base: '/' + (process.env.CI_PROJECT_NAME || ''),
        },

        vite: {
            ssr: {
                noExternal: ['feather-icons'],
            },
            plugins: [
                {
                    name: 'filesize',
                    async load(id) {
                        if (/(\?|&)size(?:&|$)/.test(id)) {
                            const path = /([^?]*)(?:\?.*)?$/.exec(id)[1];
                            return `export default ${(await fs.stat(path)).size}`;
                        }
                    },
                    enforce: 'pre',
                },
                {
                    name: 'svg',
                    async load(id) {
                        const path = /([^?]*)(?:\?.*)?$/.exec(id)[1];
                        if (path.endsWith('.svg')) {
                            return await fs.readFile(path, 'utf-8');
                        }
                        return;
                    },
                    transform(source, id, ssr) {
                        return svelteSVG(ssr ? { generate: 'ssr' } : {}).transform(source, id);
                    },
                    enforce: 'pre',
                },
            ],
        },
    },
};
