import { defineConfig } from 'orval';

export default defineConfig({
  'ds-protocol': {
    input: {
      target: '../static/specs/openapi/fe-gateway.yaml',
    },
    output: {
      mode: 'tags-split',
      target: 'shared/src/data/orval/ds-protocol.ts',
      schemas: 'shared/src/data/orval/model',
      client: 'react-query',
      mock: false,
      override: {
        mutator: {
          path: 'shared/src/data/orval-mutator.ts',
          name: 'customInstance',
        },
        query: {
          useQuery: true,
          useInfinite: true,
          // This ensures compatibility with TanStack Router's loader pattern
          // by generating the queryOptions helper.
          options: {
            // staleTime: 10000, 
          }
        },
      },
    },
  },
});
