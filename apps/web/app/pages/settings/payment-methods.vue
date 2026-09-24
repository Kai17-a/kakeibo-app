<script setup lang="ts">
import type { TableColumn } from '@nuxt/ui'
import type { PaymentMethod } from '~/types/settings'
import { validateNamed, validAmount } from '~/utils/settings'
import { formatSignedCurrency } from '~/utils/format'

useSeoMeta({ title: '支払方法設定' })
const label = '支払方法'
const api = useSettingsApi().paymentMethods
const items = ref<PaymentMethod[]>([])
const deleting = ref<PaymentMethod | null>(null)
const state = reactive<{ name: string, description: string, initial_balance: string }>({ name: '', description: '', initial_balance: '' })
const sortedItems = computed(() => [...items.value].sort((a, b) => a.name.localeCompare(b.name, 'ja')))
const columns: TableColumn<PaymentMethod>[] = [
  { accessorKey: 'name', header: '支払方法名', meta: { class: { th: 'w-full min-w-0', td: 'w-full min-w-0' } } },
  { accessorKey: 'balance', header: '残高', meta: { class: { th: 'w-24 whitespace-nowrap text-right', td: 'w-24 whitespace-nowrap text-right tabular-nums' } } },
  { id: 'actions', header: '操作', meta: { class: { th: 'w-14 whitespace-nowrap text-right', td: 'w-14 whitespace-nowrap text-right' } } }
]
function validate() {
  const errors = validateNamed(state)
  if (String(state.initial_balance).trim() && !validAmount(state.initial_balance)) errors.push({ name: 'initial_balance', message: '0以上の整数で入力してください。' })
  return errors
}
function openForm(item?: PaymentMethod) {
  editingId.value = item?.id ?? null
  Object.assign(state, { name: item?.name ?? '', description: item?.description ?? '', initial_balance: item?.initial_balance ?? '' })
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
    items.value = await api.list()
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
    const input = { name: state.name.trim(), description: state.description.trim() || null, initial_balance: String(state.initial_balance).trim() || undefined }
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
function askDelete(item: PaymentMethod) {
  deleting.value = item
  deleteError.value = ''
  deleteOpen.value = true
}
function paymentMethodActions(item: PaymentMethod) {
  return [[
    { label: '編集', icon: 'i-lucide-pencil', onSelect: () => openForm(item) },
    { label: '削除', icon: 'i-lucide-trash-2', color: 'error' as const, onSelect: () => askDelete(item) }
  ]]
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
  } catch {
    deleteError.value = '支払方法を削除できませんでした。登録済みの明細で使用されている場合は削除できません。'
  } finally {
    removing.value = false
  }
}
</script>

<template>
  <section
    class="space-y-6"
    aria-labelledby="payment-method-heading"
  >
    <div class="flex min-w-0 items-start justify-between gap-3 sm:gap-4">
      <div class="min-w-0 space-y-1">
        <h2
          id="payment-method-heading"
          class="flex flex-wrap items-center gap-2 text-xl font-semibold text-highlighted"
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
          支出の登録時に選択する支払方法と残高を管理します。
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
          name="i-lucide-wallet"
          class="size-8 text-muted"
        />
        <h3 class="font-semibold">
          {{ label }}がありません
        </h3>
        <p class="text-sm text-muted">
          利用する支払方法と開始時点の残高を登録すると、ここに一覧が表示されます。
        </p>
        <UButton
          color="neutral"
          variant="outline"
          icon="i-lucide-plus"
          @click="openForm()"
        >
          支払方法を追加
        </UButton>
      </div>
    </UCard>
    <div
      v-else
      class="min-w-0 overflow-hidden rounded-lg border border-default"
    >
      <UTable
        :data="sortedItems"
        :columns="columns"
        :aria-label="`${label}一覧`"
        class="w-full"
        :ui="{ root: 'overflow-hidden', base: 'w-full table-fixed', th: 'px-3 py-2 sm:px-4', td: 'px-3 py-2 whitespace-normal sm:px-4' }"
      >
        <template #name-cell="{ row }">
          <div class="min-w-0">
            <p class="whitespace-normal break-words font-medium text-default">
              {{ row.original.name }}
            </p>
            <p
              v-if="row.original.description"
              class="mt-0.5 whitespace-pre-wrap break-words text-xs text-muted"
            >
              {{ row.original.description }}
            </p>
          </div>
        </template>
        <template #balance-cell="{ row }">
          <span
            v-if="row.original.balance === null"
            class="text-muted"
          >未設定</span>
          <span
            v-else
            :class="Number(row.original.balance) > 0 ? 'text-primary' : Number(row.original.balance) < 0 ? 'text-error' : 'text-default'"
          >
            {{ formatSignedCurrency(row.original.balance) }}
          </span>
        </template>
        <template #actions-cell="{ row }">
          <UDropdownMenu :items="paymentMethodActions(row.original)">
            <UButton
              icon="i-lucide-ellipsis"
              color="neutral"
              variant="ghost"
              size="sm"
              :aria-label="`${row.original.name}の操作`"
            />
          </UDropdownMenu>
        </template>
      </UTable>
    </div>
    <UModal
      v-model:open="formOpen"
      :title="`${label}を${editingId ? '編集' : '追加'}`"
      description="支払方法名、初期残高、説明を設定します。"
      :dismissible="!saving"
      :close="!saving"
      :ui="{ footer: 'justify-end' }"
    >
      <template #body>
        <UForm
          id="payment-method-form"
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
          <UFormField
            name="initial_balance"
            label="初期残高（任意）"
            description="残高管理を行う場合の開始時点の金額を入力してください。"
          >
            <UInput
              :model-value="state.initial_balance"
              type="number"
              :min="0"
              :step="1"
              class="w-full"
              @update:model-value="state.initial_balance = String($event ?? '')"
            />
          </UFormField>
          <UFormField
            name="name"
            label="支払方法名"
            required
          >
            <UInput
              v-model="state.name"
              :maxlength="100"
              class="w-full"
              autofocus
            />
          </UFormField>
          <UFormField
            name="description"
            label="説明（任意）"
          >
            <UTextarea
              v-model="state.description"
              :maxlength="500"
              class="w-full"
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
          form="payment-method-form"
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
