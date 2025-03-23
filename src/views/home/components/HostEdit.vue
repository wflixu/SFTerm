<template>
    <div class="edit-container">
        <div class="edit-section">
            <h3>Address</h3>
            <div class="address-input">
                <div class="address-icon">
                    <i class="pi pi-server"></i>
                </div>
                <InputText v-model="host.address" placeholder="IP or Hostname" class="address-field" />
            </div>
        </div>

        <div class="edit-section">
            <h3>General</h3>
            <div class="form-field">
                <InputText v-model="host.name" placeholder="Label" class="full-width" />
            </div>
            <div class="form-field">
                <InputText v-model="host.desc" placeholder="Description" class="full-width" />
            </div>

        </div>

        <div class="form-actions">
            <Button label="Cancel" class="p-button-text" />
            <Button label="Save" severity="success" @click="save" />
        </div>
    </div>
</template>

<script setup lang="ts">
import type { IHost } from '@/shared/interface';
import { reactive } from 'vue';

const props = defineProps<{
    host: IHost
}>();

const emits = defineEmits<{
    (event: 'submit', host: IHost): void;
}>()

const host = reactive<IHost>({
    address: '',
    name: '',
    desc: '',
    id: new Date().getTime().toString() as string
});

if(props.host) {
    host.address = props.host.address;
    host.name = props.host.name;
    host.desc = props.host.desc;
    host.id = props.host.id;
}

const save = () => {
    emits('submit', host);
}

</script>

<style scoped>
.edit-container {
    padding: 20px;
}

.edit-section {
    margin-bottom: 20px;
    background-color: #f8f9fa;
    border-radius: 8px;
    padding: 16px;
}

.edit-section h3 {
    margin-top: 0;
    margin-bottom: 16px;
    font-size: 16px;
    font-weight: 600;
    color: #333;
}

.address-input {
    display: flex;
    align-items: center;
    gap: 12px;
}

.address-icon {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 40px;
    height: 40px;
    border-radius: 8px;
    background-color: #2c3e50;
}

.address-icon i {
    font-size: 20px;
    color: white;
}

.address-field {
    flex: 1;
}

.form-field {
    margin-bottom: 12px;
}

.form-field:last-child {
    margin-bottom: 0;
}

.input-with-icon {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 8px 12px;
    border: 1px solid #ced4da;
    border-radius: 4px;
    background-color: white;
}

.input-with-icon i {
    color: #6c757d;
    font-size: 16px;
}

.space-between {
    justify-content: space-between;
}

.default-label {
    color: #6c757d;
    font-size: 14px;
}

.full-width {
    width: 100%;
}

.form-actions {
    display: flex;
    justify-content: flex-end;
    gap: 12px;
    margin-top: 20px;
}
</style>