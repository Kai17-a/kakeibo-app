<script setup lang="ts">
import type { TableColumn } from '@nuxt/ui'
import type { Category, PaymentMethod, RecurringExpense } from '~/types/settings'
import { formatYen } from '~/utils/settings'
import { monthLabel, recurringExpenseForm, recurringExpenseInput, validateRecurringExpense } from '~/utils/recurring-expenses'

useSeoMeta({ title: '定期支出設定' })
const label = '定期支出'
const settingsApi = useSettingsApi()
const api = settingsApi.recurringExpenses
const items = ref<RecurringExpense[]>([])
const categories = ref<Category[]>([])
const paymentMethods = ref<PaymentMethod[]>([])
const pendingMonths = ref<Record<string, string[]>>({})
const backfilling = ref<Record<string, boolean>>({})
const deleting = ref<RecurringExpense | null>(null)
const state = reactive(recurringExpenseForm())
const categoryOptions = computed(() => categories.value.map(item => ({ label: item.name, value: item.id })))
const paymentMethodOptions = computed(() => paymentMethods.value.map(item => ({ label: item.name, value: item.id })))
const sortedItems = computed(() => [...items.value].sort((a, b) => a.name.localeCompare(b.name, 'ja')))
const columns: TableColumn<RecurringExpense>[] = [
  { accessorKey: 'name', header: '名称' },
  { id: 'category', header: 'カテゴリ / 支払方法' },
  { accessorKey: 'payment_day', header: '支払日', cell: ({ row }) => `毎月${row.original.payment_day}日` },
  { accessorKey: 'amount', header: '金額', cell: ({ row }) => formatYen(row.original.amount), meta: { class: { th: 'text-right', td: 'text-right tabular-nums' } } },
  { accessorKey: 'is_active', header: '状態' },
  { id: 'actions', header: '操作' }
]

function categoryName(id: string) {
  return categories.value.find(item => item.id === id)?.name ?? '不明なカテゴリ'
}
function paymentMethodName(id: string) {
  return paymentMethods.value.find(item => item.id === id)?.name ?? '不明な支払方法'
}
function validate() {
  return validateRecurringExpense(state, categories.value, paymentMethods.value)
}
function openForm(item?: RecurringExpense) {
  editingId.value = item?.id ?? null
  Object.assign(state, recurringExpenseForm(item))
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

async function loadPendingMonths(list: RecurringExpense[]) {
  const entries = await Promise.all(
    list.map(async item => [item.id, (await api.pendingMonths(item.id)).months] as const)
  )
  pendingMonths.value = Object.fromEntries(entries)
}

async function load() {
  loading.value = true
  loadError.value = ''
  try {
    const [expenses, expenseCategories, methods] = await Promise.all([
      api.list(),
      settingsApi.categories('expense').list(),
      settingsApi.paymentMethods.list()
    ])
    items.value = expenses
    categories.value = expenseCategories
    paymentMethods.value = methods
    await loadPendingMonths(expenses)
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
    const input = recurringExpenseInput(state)
    const result = editingId.value ? await api.update(editingId.value, input) : await api.create(input)
    items.value = editingId.value ? items.value.map(item => item.id === result.id ? result : item) : [...items.value, result]
    formOpen.value = false
    toast.add({ title: `${label}を${editingId.value ? '更新' : '追加'}しました`, color: 'success' })
    await loadPendingMonths(items.value)
  } catch (error) {
    formError.value = apiErrorMessage(error)
  } finally {
    saving.value = false
  }
}
function askDelete(item: RecurringExpense) {
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
async function backfill(item: RecurringExpense) {
  if (backfilling.value[item.id]) return
  backfilling.value = { ...backfilling.value, [item.id]: true }
  try {
    const result = await api.backfill(item.id)
    toast.add({
      title: result.created.length > 0 ? `${result.created.length}か月分を計上しました` : '計上対象の月はありませんでした',
      color: 'success'
    })
    await loadPendingMonths(items.value)
  } catch (error) {
    toast.add({ title: '過去分を計上できませんでした', description: apiErrorMessage(error), color: 'error' })
  } finally {
    backfilling.value = { ...backfilling.value, [item.id]: false }
  }
}
</script>

<template>
  <section
    class="space-y-6"
    aria-labelledby="recurring-expense-heading"
  >
    <div class="flex items-start justify-between gap-4">
      <div class="space-y-2">
        <h2
          id="recurring-expense-heading"
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
          毎月発生する固定費・準固定費（金額変動）の支出予定を管理します。
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
          右上の「追加」から定期支出を登録してください。
        </p>
      </div>
    </UCard>
    <template v-else>
      <UTable
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
              準固定費
            </UBadge>
          </div>
        </template>
        <template #category-cell="{ row }">
          <span class="text-sm text-muted whitespace-normal break-words">
            {{ categoryName(row.original.category_id) }} / {{ paymentMethodName(row.original.payment_method_id) }}
          </span>
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
      <div class="flex flex-col gap-3">
        <template
          v-for="item in sortedItems"
          :key="item.id"
        >
          <div
            v-if="(pendingMonths[item.id]?.length ?? 0) > 0"
            class="flex flex-wrap items-center gap-3 rounded-md bg-elevated/50 px-4 py-3 text-sm"
          >
            <span class="font-medium">{{ item.name }}</span>
            <UBadge
              color="neutral"
              variant="soft"
            >
              未計上 {{ pendingMonths[item.id]?.length }}件
            </UBadge>
            <UButton
              v-if="!item.is_variable"
              color="neutral"
              variant="outline"
              size="sm"
              :loading="backfilling[item.id]"
              :disabled="backfilling[item.id]"
              @click="backfill(item)"
            >
              まとめて計上する
            </UButton>
            <details
              v-else
              class="w-full"
            >
              <summary class="cursor-pointer text-muted">
                過去月を登録
              </summary>
              <div class="mt-2 flex flex-wrap gap-x-4 gap-y-1">
                <a
                  v-for="month in pendingMonths[item.id]"
                  :key="month"
                  class="text-primary underline underline-offset-4"
                  :href="`/monthly?month=${month}`"
                >
                  {{ monthLabel(month) }}
                </a>
              </div>
            </details>
          </div>
        </template>
      </div>
    </template>

    <UModal
      v-model:open="formOpen"
      :title="`${label}を${editingId ? '編集' : '追加'}`"
      description="毎月の支払額、支払日、適用期間を設定します。"
      :dismissible="!saving"
      :close="!saving"
      :ui="{ footer: 'justify-end', content: 'max-w-2xl' }"
    >
      <template #body>
        <UForm
          id="recurring-expense-form"
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
            v-if="!categories.length || !paymentMethods.length"
            color="warning"
            title="カテゴリ・支払方法が不足しています"
            description="設定の「支出カテゴリ」「支払方法」で登録してから追加してください。"
          />
          <UFormField
            name="name"
            label="名称"
            required
          >
            <UInput
              v-model="state.name"
              class="w-full"
            />
          </UFormField>
          <UFormField name="usdBased">
            <UCheckbox
              v-model="state.usdBased"
              label="外貨建て（USD）にする"
            />
          </UFormField>
          <UFormField
            v-if="!state.usdBased"
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
            v-else
            name="foreignAmount"
            label="毎月のUSD金額"
            description="実際の引き落とし額は毎月の為替レートで自動計算されます。"
            required
          >
            <UInput
              :model-value="state.foreignAmount"
              type="number"
              :min="0.01"
              step="any"
              class="w-full"
              @update:model-value="state.foreignAmount = String($event ?? '')"
            />
          </UFormField>
          <UFormField
            name="payment_day"
            label="毎月の支払日"
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
            name="payment_method_id"
            label="支払方法"
            required
          >
            <USelect
              v-model="state.payment_method_id"
              :items="paymentMethodOptions"
              placeholder="選択してください"
              class="w-full"
            />
          </UFormField>
          <UFormField
            name="start_date"
            label="開始日"
            required
          >
            <DatePicker
              v-model="state.start_date"
              required
              class="w-full"
            />
          </UFormField>
          <UFormField
            name="end_date"
            label="終了日（任意）"
          >
            <DatePicker
              v-model="state.end_date"
              :min="state.start_date"
              clearable
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
              label="金額が月ごとに変動する（準固定費）"
            />
          </UFormField>
          <UFormField
            v-if="editingId"
            name="sync_future_transactions"
          >
            <UCheckbox
              v-model="state.sync_future_transactions"
              label="今月以降に生成済みの明細へ金額・カテゴリ・支払方法・備考を反映する"
            />
          </UFormField>
          <UFormField name="is_active">
            <UCheckbox
              v-model="state.is_active"
              :label="editingId ? '有効にする' : '登録後すぐに有効にする'"
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
          form="recurring-expense-form"
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
