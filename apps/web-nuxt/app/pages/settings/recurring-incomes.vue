<script setup lang="ts">
import type { TableColumn } from '@nuxt/ui'
import type { Category, RecurringIncome } from '~/types/settings'
import { formatYen } from '~/utils/settings'
import { recurringIncomeForm, recurringIncomeInput, validateRecurringIncome } from '~/utils/recurring-incomes'

useSeoMeta({ title: '定期収入設定' })
const label = '定期収入'
const settingsApi = useSettingsApi()
const api = settingsApi.recurringIncomes
const categories = ref<Category[]>([])
const categoryOptions = computed(() => categories.value.map(item => ({ label: item.name, value: item.id })))
const items = ref<RecurringIncome[]>([])
const deleting = ref<RecurringIncome | null>(null)
const state = reactive(recurringIncomeForm())
const sortedItems = computed(() => [...items.value].sort((a, b) => a.name.localeCompare(b.name, 'ja')))
const columns: TableColumn<RecurringIncome>[] = [
  { accessorKey: 'name', header: '名称' },
  { accessorKey: 'category_id', header: 'カテゴリ名', cell: ({ row }) => categories.value.find(item => item.id === row.original.category_id)?.name ?? '不明なカテゴリ' },
  { accessorKey: 'payment_day', header: '入金日', cell: ({ row }) => `毎月${row.original.payment_day}日` },
  { accessorKey: 'amount', header: '金額', cell: ({ row }) => formatYen(row.original.amount), meta: { class: { th: 'text-right', td: 'text-right tabular-nums' } } },
  { accessorKey: 'is_active', header: '状態' },
  { id: 'actions', header: '操作' }
]
function validate() {
  return validateRecurringIncome(state, categories.value)
}

function openForm(item?: RecurringIncome) {
  editingId.value = item?.id ?? null
  Object.assign(state, recurringIncomeForm(item))
  formError.value = ''
  formOpen.value = true
}

const toast = useToast()
const loading = ref(true)
const loadError = ref('')
const formOpen = ref(false)
const deleteOpen = ref(false)
const editingId = ref<string | null>(null)
const saving = ref(false)
const removing = ref(false)
const formError = ref('')
const deleteError = ref('')

async function load() {
  loading.value = true
  loadError.value = ''
  try {
    const [incomes, incomeCategories] = await Promise.all([api.list(), settingsApi.categories('income').list()])
    items.value = incomes
    categories.value = incomeCategories
  } catch (error) {
    loadError.value = apiErrorMessage(error)
  } finally {
    loading.value = false
  }
}
onMounted(load)

async function save() {
  if (saving.value) return
  saving.value = true
  formError.value = ''
  try {
    const input = recurringIncomeInput(state)
    const result = editingId.value ? await api.update(editingId.value, input) : await api.create(input)
    items.value = editingId.value ? items.value.map(item => item.id === result.id ? result : item) : [...items.value, result]
    formOpen.value = false
    toast.add({ title: `${label}を${editingId.value ? '更新' : '追加'}しました`, color: 'success' })
  } catch (error) {
    formError.value = apiErrorMessage(error)
  } finally {
    saving.value = false
  }
}
function askDelete(item: RecurringIncome) {
  deleting.value = item
  deleteError.value = ''
  deleteOpen.value = true
}
async function remove() {
  if (!deleting.value || removing.value) return
  removing.value = true
  deleteError.value = ''
  const id = deleting.value.id
  try {
    await api.remove(id)
    items.value = items.value.filter(item => item.id !== id)
    deleteOpen.value = false
    toast.add({ title: `${label}を削除しました`, color: 'success' })
  } catch (error) {
    deleteError.value = apiErrorMessage(error)
  } finally {
    removing.value = false
  }
}
</script>

<template>
  <section
    class="space-y-6"
    aria-labelledby="recurring-income-heading"
  >
    <div class="flex items-start justify-between gap-4">
      <div class="space-y-2">
        <h2
          id="recurring-income-heading"
          class="text-lg font-semibold flex items-center gap-3"
        >
          {{ label }}
          <UBadge
            v-if="!loading && !loadError"
            color="neutral"
            variant="soft"
          >
            {{ items.length }}件
          </UBadge>
        </h2>
        <p class="text-sm text-muted">
          毎月発生する固定収入・準固定収入（金額変動）の入金予定を管理します。
        </p>
      </div>
      <UButton
        icon="i-lucide-plus"
        class="shrink-0"
        :disabled="loading || !!loadError"
        @click="openForm()"
      >
        追加
      </UButton>
    </div>

    <div
      v-if="loading"
      role="status"
      :aria-label="`${label}を読み込み中`"
      class="space-y-3"
    >
      <USkeleton
        v-for="i in 3"
        :key="i"
        class="h-16 w-full"
      />
      <span class="sr-only">読み込み中…</span>
    </div>
    <UAlert
      v-else-if="loadError"
      color="error"
      :title="`${label}を取得できませんでした`"
      :description="loadError"
      :actions="[{ label: '再試行', color: 'error', variant: 'outline', onClick: load }]"
    />
    <UCard
      v-else-if="!items.length"
      class="text-center"
    >
      <div class="py-10 space-y-3">
        <UIcon
          name="i-lucide-repeat"
          class="size-8 text-muted"
        />
        <h3 class="font-semibold">
          {{ label }}がありません
        </h3>
        <p class="text-sm text-muted">
          右上の「追加」から定期収入を登録してください。
        </p>
      </div>
    </UCard>
    <UTable
      v-else
      :data="sortedItems"
      :columns="columns"
      :aria-label="`${label}一覧`"
    >
      <template #name-cell="{ row }">
        <div class="flex flex-wrap items-center gap-2">
          <span class="whitespace-normal break-words">{{ row.original.name }}</span>
          <UBadge
            v-if="row.original.is_variable"
            color="neutral"
            variant="soft"
          >
            準固定収入
          </UBadge>
        </div>
      </template>
      <template #is_active-cell="{ row }">
        <UBadge
          :color="row.original.is_active ? 'success' : 'neutral'"
          variant="soft"
        >
          {{ row.original.is_active ? '有効' : '無効' }}
        </UBadge>
      </template>
      <template #actions-cell="{ row }">
        <div class="flex gap-2">
          <UButton
            color="neutral"
            variant="ghost"
            :aria-label="`${row.original.name}を編集`"
            @click="openForm(row.original)"
          >
            編集
          </UButton>
          <UButton
            color="error"
            variant="soft"
            :aria-label="`${row.original.name}を削除`"
            @click="askDelete(row.original)"
          >
            削除
          </UButton>
        </div>
      </template>
    </UTable>
    <UModal
      v-model:open="formOpen"
      :title="`${label}を${editingId ? '編集' : '追加'}`"
      description="毎月の収入額、入金日、適用期間を設定します。"
      :dismissible="!saving"
      :close="!saving"
      :ui="{ footer: 'justify-end' }"
    >
      <template #body>
        <UForm
          id="recurring-income-form"
          :state="state"
          :validate="validate"
          :disabled="saving"
          class="space-y-5"
          @submit="save"
        >
          <UAlert
            v-if="formError"
            color="error"
            title="保存できませんでした"
            :description="formError"
          />
          <UAlert
            v-if="!categories.length"
            color="warning"
            title="収入カテゴリがありません"
            description="設定の「収入カテゴリ」でカテゴリを登録してから追加してください。"
          />
          <UFormField
            name="name"
            label="名称"
            required
          >
            <UInput
              v-model="state.name"
              class="w-full"
              autofocus
            />
          </UFormField>
          <UFormField
            name="amount"
            :label="state.is_variable ? '金額（目安）' : '金額'"
            required
          >
            <UInput
              :model-value="state.amount"
              type="number"
              :min="1"
              :step="1"
              class="w-full"
              @update:model-value="state.amount = String($event ?? '')"
            />
          </UFormField>
          <UFormField
            name="payment_day"
            label="毎月の入金日"
            required
          >
            <UInput
              :model-value="state.payment_day"
              type="number"
              :min="1"
              :max="31"
              :step="1"
              class="w-full"
              @update:model-value="state.payment_day = String($event ?? '')"
            />
          </UFormField>
          <UFormField
            name="category_id"
            label="カテゴリ"
            required
          >
            <USelect
              v-model="state.category_id"
              :items="categoryOptions"
              placeholder="選択してください"
              class="w-full"
            />
          </UFormField>
          <UFormField
            name="start_date"
            label="開始日"
            required
          >
            <UInput
              v-model="state.start_date"
              type="date"
              class="w-full"
            />
          </UFormField>
          <UFormField
            name="end_date"
            label="終了日（任意）"
          >
            <UInput
              v-model="state.end_date"
              type="date"
              :min="state.start_date || undefined"
              class="w-full"
            />
          </UFormField>
          <UFormField
            name="description"
            label="備考（任意）"
          >
            <UTextarea
              v-model="state.description"
              class="w-full"
            />
          </UFormField>
          <UFormField name="is_variable">
            <UCheckbox
              v-model="state.is_variable"
              label="金額が月ごとに変動する（準固定収入）"
            />
          </UFormField>
          <UFormField name="is_active">
            <UCheckbox
              v-model="state.is_active"
              label="有効にする"
            />
          </UFormField>
        </UForm>
      </template>
      <template #footer>
        <UButton
          color="neutral"
          variant="outline"
          :disabled="saving"
          @click="formOpen = false"
        >
          キャンセル
        </UButton>
        <UButton
          type="submit"
          form="recurring-income-form"
          :loading="saving"
          :disabled="saving"
        >
          {{ editingId ? '更新' : '追加' }}
        </UButton>
      </template>
    </UModal>
    <UModal
      v-model:open="deleteOpen"
      :title="`${label}を削除`"
      description="この設定を削除します。この操作は取り消せません。"
      :dismissible="!removing"
      :close="!removing"
      :ui="{ footer: 'justify-end' }"
    >
      <template #body>
        <p class="break-all">
          {{ deleting?.name }}
        </p>
        <UAlert
          v-if="deleteError"
          color="error"
          title="削除できませんでした"
          :description="deleteError"
          class="mt-4"
        />
      </template>
      <template #footer>
        <UButton
          color="neutral"
          variant="outline"
          :disabled="removing"
          @click="deleteOpen = false"
        >
          キャンセル
        </UButton>
        <UButton
          color="error"
          :loading="removing"
          :disabled="removing"
          @click="remove"
        >
          削除
        </UButton>
      </template>
    </UModal>
  </section>
</template>
