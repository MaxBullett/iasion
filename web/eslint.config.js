// Lint for web/ (ADR-0043, decision 4). Prettier formats; ESLint does not.
import pluginVue from 'eslint-plugin-vue'
import pluginVueA11y from 'eslint-plugin-vuejs-accessibility'
import prettier from 'eslint-config-prettier/flat'
import { withVueTs, vueTsConfigs } from '@vue/eslint-config-typescript'

export default withVueTs(
  { ignores: ['dist/', 'src/api/generated/', 'playwright-report/', 'test-results/'] },
  pluginVue.configs['flat/recommended'],
  pluginVueA11y.configs['flat/recommended'],
  vueTsConfigs.recommended,
  prettier,
)
