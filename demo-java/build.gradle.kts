plugins{
  application //声明使用哪些插件
}

repositories {
  mavenCentral()   //依赖去哪个地方找
}

dependencies{
  implementation("com.google.code.gson:gson:2.11.0") //声明依赖
}

java{
  toolchain{
    languageVersion = JavaLanguageVersion.of(27) //指定用27版本编译器
  }
}

application {
  mainClass = "demo.player.Main" //入口类，必须是全限定名：包名.类名
  applicationName="maplayer-java" //生成脚本用名
}

tasks.withType<JavaCompile>{
  options.encoding = "UTF-8" //对所有java的任务统一设置，tasks是所有任务容器。withType<T>是按类型筛选任务。javaCompile是参数类型，只挑编译任务
}

tasks.named<JavaExec>("run"){        //配置run的这个任务name<JavaExec>按照名查找任务断言类型是JavaExec
  standardInput = System.`in`  //把进程的标准输入接到run任务
}


