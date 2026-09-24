<script setup lang="ts">
import type { TableColumn } from '@nuxt/ui'
import type { Category, CategoryKind } from '~/types/settings'
import { categoryMove, groupCategories, validateNamed } from '~/utils/settings'

const props = defineProps<{ kind: CategoryKind }>()
const label = computed(() => props.kind === 'expense' ? '支出カテゴリ' : '収入カテゴリ')
const api = useSettingsApi().categories(props.kind)
const items = ref<Category[]>([])
const deleting = ref<Category | null>(null)
const reordering = ref(false)
const orderError = ref('')
const state = reactive({ name: '', description: '', parent_category_id: 'none' })
const sortedItems = computed(() => groupCategories(items.value))
const editingHasChildren = computed(() => !!editingId.value && items.value.some(item => item.parent_category_id === editingId.value))
const parentOptions = computed(() => [
  { label: '親カテゴリなし', value: 'none' },
  ...sortedItems.value.filter(item => item.parent_category_id === null && item.id !== editingId.value).map(item => ({ label: item.name, value: item.id }))
])
const columns: TableColumn<Category>[] = [
  { accessorKey: 'name', header: 'カテゴリ名' },
  { accessorKey: 'description', header: '説明' },
  { id: 'actions', header: '操作' }
]
const validate = validateNamed
function openForm(item?: Category) {
  editingId.value = item?.id ?? null
  Object.assign(state, { name: item?.name ?? '', description: item?.description ?? '', parent_category_id: item?.parent_category_id ?? 'none' })
  formError.value = ''
  formOpen.value = true
}
function canMove(item: Category, direction: -1 | 1) {
  return categoryMove(items.value, item.id, direction) !== null
}
async function move(item: Category, direction: -1 | 1) {
  if (reordering.value) return
  const result = categoryMove(items.value, item.id, direction)
  if (!result) return
  const previous = items.value
  items.value = result.items
  reordering.value = true
  orderError.value = ''
  try {
    await api.reorder(result.input)
    toast.add({ title: 'カテゴリの表示順を更新しました', color: 'success' })
  } catch (error) {
    items.value = previous
    orderError.value = apiErrorMessage(error)
  } finally {
    reordering.value = false
  }
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
    const input = { name: state.name.trim(), description: state.description.trim() || null, parent_category_id: editingHasChildren.value || state.parent_category_id === 'none' ? null : state.parent_category_id }
    const result = editingId.value ? await api.update(editingId.value, input) : await api.create(input)
    items.value = editingId.value ? items.value.map(item => item.id === result.id ? result : item) : [...items.value, result]
    formOpen.value = false
    toast.add({ title: `${label.value}を${editingId.value ? '更新' : '追加'}しました`, color: 'success' })
  } catch (error) {
    formError.value = apiErrorMessage(error)
  } finally {
    saving.value = false
  }
}
function askDelete(item: Category) {
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
    toast.add({ title: `${label.value}を削除しました`, color: 'success' })
  } catch {
    deleteError.value = 'カテゴリを削除できませんでした。登録済みの明細で使用されているか、子カテゴリが存在するため削除できません。'
  } finally {
    removing.value = false
  }
}
</script>

<template>
  <section
    class="space-y-6"
    aria-labelledby="category-heading"
  >
    <div class="flex items-start justify-between gap-4">
      <div class="space-y-2">
        <h2
          id="category-heading"
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
          収支の登録時に選択するカテゴリと表示順を管理します。
        </p>
      </div>
      <UButton
        icon="i-lucide-plus"
        class="shrink-0"
        :disabled="loading || !!loadError || reordering"
        @click="openForm()"
      >
        追加
      </UButton>
    </div>
    <UAlert
      v-if="orderError"
      color="error"
      title="表示順を更新できませんでした"
      :description="orderError"
    />
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
          name="i-lucide-tags"
          class="size-8 text-muted"
        />
        <h3 class="font-semibold">
          {{ label }}がありません
        </h3>
        <p class="text-sm text-muted">
          右上の「追加」からカテゴリを登録してください。
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
        <div :class="['flex items-center gap-2', row.original.parent_category_id !== null ? 'pl-6' : '']">
          <span class="whitespace-normal break-words">{{ row.original.name }}</span>
          <UBadge
            color="neutral"
            variant="soft"
          >
            {{ row.original.parent_category_id === null ? '親カテゴリ' : '子カテゴリ' }}
          </UBadge>
        </div>
      </template>
      <template #description-cell="{ row }">
        <p class="whitespace-pre-wrap break-words">
          {{ row.original.description || '—' }}
        </p>
      </template>
      <template #actions-cell="{ row }">
        <div class="flex gap-2">
          <UButton
            icon="i-lucide-arrow-up"
            color="neutral"
            variant="ghost"
            :aria-label="`${row.original.name}を上へ移動`"
            :disabled="reordering || !canMove(row.original, -1)"
            @click="move(row.original, -1)"
          />
          <UButton
            icon="i-lucide-arrow-down"
            color="neutral"
            variant="ghost"
            :aria-label="`${row.original.name}を下へ移動`"
            :disabled="reordering || !canMove(row.original, 1)"
            @click="move(row.original, 1)"
          />
          <UButton
            color="neutral"
            variant="ghost"
            :aria-label="`${row.original.name}を編集`"
            :disabled="reordering"
            @click="openForm(row.original)"
          >
            編集
          </UButton>
          <UButton
            color="error"
            variant="soft"
            :aria-label="`${row.original.name}を削除`"
            :disabled="reordering"
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
      description="カテゴリ名、親カテゴリ、説明を設定します。"
      :dismissible="!saving"
      :close="!saving"
      :ui="{ footer: 'justify-end' }"
    >
      <template #body>
        <UForm
          id="category-form"
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
            name="name"
            label="カテゴリ名"
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
            name="parent_category_id"
            label="親カテゴリ（任意）"
            :description="editingHasChildren ? '子カテゴリが存在するため、このカテゴリには親を設定できません。' : undefined"
          >
            <USelect
              v-model="state.parent_category_id"
              :items="parentOptions"
              :disabled="editingHasChildren"
              class="w-full"
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
          form="category-form"
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
