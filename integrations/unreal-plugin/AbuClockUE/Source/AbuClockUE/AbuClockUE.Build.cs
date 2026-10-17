using UnrealBuildTool;
using System.IO;

public class AbuClockUE : ModuleRules
{
    public AbuClockUE(ReadOnlyTargetRules Target) : base(Target)
    {
        PCHUsage = ModuleRules.PCHUsageMode.UseExplicitOrSharedPCHs;

        PublicIncludePaths.AddRange(new string[] {
            Path.Combine(ModuleDirectory, "../../../../crates/abu-ffi/include")
        });

        PublicDependencyModuleNames.AddRange(new string[] {
            "Core",
            "CoreUObject",
            "Engine",
            "TimeManagement",
            "LiveLinkInterface",
            "Networking",
            "Sockets"
        });
    }
}
