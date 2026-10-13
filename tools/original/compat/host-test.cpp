// Public host ABI contracts. This does not inspect original gameplay code.
#include "steam_api.h"
#include <cmath>
#include <cstring>
#include <iostream>
#include <limits>
#include <stdexcept>
#include <string>

namespace {
int checks=0;
void require(bool condition,const char*message){++checks;if(!condition)throw std::runtime_error(message);}
struct Callback final:CCallbackBase{
  int kind,count=0;
  explicit Callback(int id):kind(id){m_iCallback=id;SteamAPI_RegisterCallback(this,id);}
  ~Callback(){SteamAPI_UnregisterCallback(this);}
  void Run(void*value)override{
    ++count;
    if(kind==UserStatsReceived_t::k_iCallback){auto*event=static_cast<UserStatsReceived_t*>(value);require(event->m_nGameID==250900&&event->m_eResult==k_EResultOK,"received callback payload");}
    if(kind==UserStatsStored_t::k_iCallback){auto*event=static_cast<UserStatsStored_t*>(value);require(event->m_nGameID==250900&&event->m_eResult==k_EResultOK,"stored callback payload");}
    if(kind==UserAchievementStored_t::k_iCallback){auto*event=static_cast<UserAchievementStored_t*>(value);require(event->m_nGameID==250900&&!event->m_bGroupAchievement&&event->m_nCurProgress==0&&event->m_nMaxProgress==0,"achievement callback payload");require(std::strcmp(event->m_rgchAchievementName,"unlock-one")==0,"only newly stored achievement callback");}
  }
  void Run(void*,bool,SteamAPICall_t)override{require(false,"unexpected call-result callback");}
  int GetCallbackSizeBytes()override{
    if(kind==UserStatsReceived_t::k_iCallback)return sizeof(UserStatsReceived_t);
    if(kind==UserStatsStored_t::k_iCallback)return sizeof(UserStatsStored_t);
    return sizeof(UserAchievementStored_t);
  }
};
constexpr char payload[]={'s','a','v','e',0,'b','y','t','e','s'};
void writePhase(){
  auto*stats=SteamUserStats();auto*files=SteamRemoteStorage();
  Callback received(UserStatsReceived_t::k_iCallback),stored(UserStatsStored_t::k_iCallback),achievement(UserAchievementStored_t::k_iCallback);
  require(stats->RequestCurrentStats(),"request stats accepted");require(received.count==0,"received callback asynchronous");
  SteamAPI_RunCallbacks();require(received.count==1,"received callback delivered once");SteamAPI_RunCallbacks();require(received.count==1,"received callback consumed");
  int32 integer=77;float real=77;bool achieved=true;
  require(stats->GetStat("absent-int",&integer)&&integer==0,"absent integer default");
  require(stats->GetStat("absent-float",&real)&&real==0,"absent float default");
  require(stats->GetAchievement("absent-achievement",&achieved)&&!achieved,"absent achievement default");
  require(stats->SetStat("counter",int32(23)),"set integer stat");
  require(stats->SetStat("negative",std::numeric_limits<int32>::min()),"set signed integer boundary");
  require(stats->SetStat("fraction",0.1f),"set float stat");
  require(stats->SetStat("signed-zero",-0.0f),"set signed zero float");
  require(!stats->SetStat("counter",1.0f),"reject conflicting float setter");
  require(!stats->SetStat("fraction",int32(1)),"reject conflicting integer setter");
  require(!stats->GetStat("counter",&real),"reject conflicting float getter");
  require(!stats->GetStat("fraction",&integer),"reject conflicting integer getter");
  require(!stats->SetStat("bad-float",std::numeric_limits<float>::infinity()),"reject infinite float");
  require(!stats->SetStat("bad-float",std::numeric_limits<float>::quiet_NaN()),"reject NaN float");
  require(!stats->SetStat("line\nbreak",int32(1)),"reject newline name");
  require(!stats->GetStat(nullptr,&integer)&&!stats->GetStat("counter",static_cast<int32*>(nullptr)),"reject null stats arguments");
  require(stats->SetAchievement("unlock-one")&&stats->SetAchievement("unlock-one"),"set achievement idempotently");
  require(stats->SetAchievement("unlock-cleared")&&stats->ClearAchievement("unlock-cleared"),"clear achievement before storing");
  require(stats->GetAchievement("unlock-one",&achieved)&&achieved,"achievement available in memory");
  require(stats->GetNumAchievements()==2,"local achievement names retained");
  require(std::strcmp(stats->GetAchievementName(0),"unlock-cleared")==0&&std::strcmp(stats->GetAchievementName(1),"unlock-one")==0,"local achievement enumeration deterministic");
  require(stats->StoreStats(),"persist stats");require(stored.count==0&&achievement.count==0,"store callbacks asynchronous");
  SteamAPI_RunCallbacks();require(stored.count==1&&achievement.count==1,"store and achievement callbacks delivered");
  SteamAPI_RunCallbacks();require(stored.count==1&&achievement.count==1,"store callbacks consumed");
  require(files->FileWrite("profile.dat",payload,sizeof(payload)),"local file write");
  require(files->FileExists("profile.dat")&&files->FilePersisted("profile.dat"),"local file exists");
  require(files->GetFileSize("profile.dat")==sizeof(payload),"local file size");
  require(!files->FileWrite("../escape",payload,sizeof(payload)),"reject local file traversal");
  require(stats->SetStat("counter",int32(999)),"unsaved stat change");
}
void readPhase(){
  auto*stats=SteamUserStats();auto*files=SteamRemoteStorage();
  Callback stored(UserStatsStored_t::k_iCallback);
  int32 integer=0;float real=0;bool achieved=false;
  require(stats->GetStat("counter",&integer)&&integer==23,"fresh process restores explicitly stored value");
  require(stats->GetStat("negative",&integer)&&integer==std::numeric_limits<int32>::min(),"fresh process restores signed integer");
  require(stats->GetStat("fraction",&real)&&std::memcmp(&real,&(const float&)(0.1f),sizeof(real))==0,"fresh process restores float bits");
  require(stats->GetStat("signed-zero",&real)&&real==0&&std::signbit(real),"fresh process restores negative zero");
  require(!stats->GetStat("fraction",&integer)&&!stats->GetStat("counter",&real),"fresh process restores stat types");
  require(stats->GetAchievement("unlock-one",&achieved)&&achieved,"fresh process restores achievement");
  require(stats->GetAchievement("unlock-cleared",&achieved)&&!achieved,"fresh process restores cleared achievement");
  uint32 unlockTime=123;
  require(stats->GetAchievementAndUnlockTime("unlock-one",&achieved,&unlockTime)&&achieved&&unlockTime==0,"offline unlock time has no remote timestamp");
  char buffer[sizeof(payload)]{};
  require(files->FileRead("profile.dat",buffer,sizeof(buffer))==sizeof(payload)&&std::memcmp(buffer,payload,sizeof(payload))==0,"fresh process reads exact local file bytes");
  require(files->FileRead("profile.dat",buffer,4)==4&&std::memcmp(buffer,payload,4)==0,"partial local file read");
  require(files->FileDelete("profile.dat")&&!files->FileExists("profile.dat"),"local file deletion");
  require(stats->ResetAllStats(false),"reset stats preserving achievements");
  require(stats->GetStat("counter",&integer)&&integer==0&&stats->GetStat("fraction",&real)&&real==0,"reset zeroes typed stats");
  require(!stats->GetStat("counter",&real)&&!stats->GetStat("fraction",&integer),"reset preserves stat types");
  require(stats->GetAchievement("unlock-one",&achieved)&&achieved,"reset false retains achievement");
  require(stats->StoreStats(),"persist partial reset");SteamAPI_RunCallbacks();require(stored.count==1,"partial reset stored callback");
  require(SteamAPI_Init(),"reload saved reset on init");
  require(stats->GetStat("counter",&integer)&&integer==0&&stats->GetAchievement("unlock-one",&achieved)&&achieved,"initialization reloads partial reset");
  require(stats->ResetAllStats(true)&&stats->StoreStats(),"reset and store achievements too");SteamAPI_RunCallbacks();require(stored.count==2,"full reset stored callback");
}
void resetReadPhase(){
  auto*stats=SteamUserStats();int32 integer=1;float real=1;bool achieved=true;
  require(stats->GetStat("counter",&integer)&&integer==0,"fresh process restores reset integer");
  require(stats->GetStat("fraction",&real)&&real==0,"fresh process restores reset float");
  require(!stats->GetStat("counter",&real)&&!stats->GetStat("fraction",&integer),"fresh reset process preserves types");
  require(stats->GetAchievement("unlock-one",&achieved)&&!achieved,"fresh process restores reset achievement");
  require(stats->GetNumAchievements()==2,"reset preserves achievement names");
  require(!SteamRemoteStorage()->FileExists("profile.dat"),"fresh process observes deleted local file");
}
}
int main(int argc,char**argv){
  try{
    require(argc==2,"phase argument required");require(SteamAPI_Init(),"initialize local host services");
    std::string phase=argv[1];
    if(phase=="write")writePhase();else if(phase=="read")readPhase();else if(phase=="reset-read")resetReadPhase();else require(false,"unknown phase");
    SteamAPI_Shutdown();
    std::cout<<"{\"phase\":\""<<phase<<"\",\"passed\":true,\"assertions\":"<<checks<<"}\n";
    return 0;
  }catch(const std::exception&error){std::cerr<<error.what()<<'\n';std::cout<<"{\"passed\":false,\"error\":\"host service assertion failed\"}\n";return 1;}
}
