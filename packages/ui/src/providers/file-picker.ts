import { createContext } from '.'

export interface PickedFile {
	/** Browser File object */
	file: File
	/** Native file system path (available on Tauri, undefined on web) */
	path?: string
	/** URL suitable for display (blob URL on web, convertFileSrc URL on Tauri) */
	previewUrl: string
}

export interface PickedModpackFile extends Omit<PickedFile, 'file'> {
	/** Only present for upload flows; native imports avoid copying huge packs into the webview heap. */
	file?: File
}

export interface PickModpackFileOptions {
	/** Set to false when a native path can be streamed directly by the backend. */
	readFile?: boolean
}

export interface FilePickerProvider {
	/** Pick an image file (for icons) */
	pickImage: () => Promise<PickedFile | null>
	/** Pick a .mrpack modpack file */
	pickModpackFile: (options?: PickModpackFileOptions) => Promise<PickedModpackFile | null>
}

export const [injectFilePicker, provideFilePicker] = createContext<FilePickerProvider>('FilePicker')
