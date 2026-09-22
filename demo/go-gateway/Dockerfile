# 构建阶段：静态编译网关（无 CGO 依赖）
FROM golang:1.27-alpine AS build
WORKDIR /src
COPY go.mod ./
COPY main.go ./
RUN CGO_ENABLED=0 go build -trimpath -ldflags "-s -w" -o /out/gateway .

# 运行阶段：scratch 空镜像 + 从构建阶段带入 CA 证书（访问上游 HTTPS 必需）
FROM scratch
COPY --from=build /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/ca-certificates.crt
COPY --from=build /out/gateway /gateway
EXPOSE 8080
ENV PORT=8080
USER 65534:65534
ENTRYPOINT ["/gateway"]
