package objectstorage

import (
	"bytes"
	"context"
	"errors"
	"fmt"
	"io"
	"net/url"
	"os"
	"path/filepath"
	"time"

	"go.std/config"

	"github.com/minio/minio-go/v7"
	"github.com/minio/minio-go/v7/pkg/credentials"
)

// A simple abstraction for object storage
type ObjectStorage struct {
	c *config.ObjectStorageConfig

	// If s3-like
	minio *minio.Client

	// if s3-like
	cdnMinio *minio.Client
}

func New(c *config.ObjectStorageConfig) (o *ObjectStorage, err error) {
	o = &ObjectStorage{
		c: c,
	}

	switch c.Type {
	case "s3-like":
		o.minio, err = minio.New(c.Endpoint, &minio.Options{
			Creds:  credentials.NewStaticV4(c.AccessKey, c.SecretKey, ""),
			Secure: c.Secure,
		})

		if err != nil {
			return nil, err
		}

		o.cdnMinio, err = minio.New(c.CdnEndpoint, &minio.Options{
			Creds:  credentials.NewStaticV4(c.AccessKey, c.SecretKey, ""),
			Secure: c.CdnSecure,
		})

		if err != nil {
			return nil, err
		}
	case "local":
		err = os.MkdirAll(c.Path, 0755)

		if err != nil {
			return nil, err
		}
	default:
		return nil, errors.New("invalid object storage type")
	}

	return o, nil
}

// Saves a file to the object storage
//
// Note that 'expiry' is not supported for local storage
func (o *ObjectStorage) Save(ctx context.Context, dir, filename string, data *bytes.Buffer, expiry time.Duration) error {
	switch o.c.Type {
	case "local":
		err := os.MkdirAll(filepath.Join(o.c.Path, dir), 0755)

		if err != nil {
			return err
		}

		f, err := os.Create(filepath.Join(o.c.Path, dir, filename))

		if err != nil {
			return err
		}

		_, err = io.Copy(f, data)

		if err != nil {
			return err
		}

		return nil
	case "s3-like":
		p := minio.PutObjectOptions{}

		if expiry != 0 {
			p.Expires = time.Now().Add(expiry)
		}
		_, err := o.minio.PutObject(ctx, o.c.Path, dir+"/"+filename, data, int64(data.Len()), p)

		if err != nil {
			return err
		}

		return nil
	default:
		return fmt.Errorf("operation not supported for object storage type %s", o.c.Type)
	}
}

// Returns the url to the file
func (o *ObjectStorage) GetUrl(ctx context.Context, dir, filename string, urlExpiry time.Duration) (*url.URL, error) {
	switch o.c.Type {
	case "local":
		var path string

		if filename == "" {
			path = filepath.Join(o.c.Path, dir)
		} else {
			path = filepath.Join(o.c.Path, dir, filename)
		}

		return &url.URL{
			Scheme: "file",
			Path:   path,
		}, nil
	case "s3-like":
		var path string

		if filename == "" {
			path = dir
		} else {
			path = dir + "/" + filename
		}

		p, err := o.cdnMinio.PresignedGetObject(ctx, o.c.Path, path, urlExpiry, nil)

		if err != nil {
			return nil, err
		}

		return p, nil
	default:
		return nil, fmt.Errorf("operation not supported for object storage type %s", o.c.Type)
	}
}

// Deletes a file
func (o *ObjectStorage) Delete(ctx context.Context, dir, filename string) error {
	switch o.c.Type {
	case "local":
		return os.Remove(filepath.Join(o.c.Path, dir, filename))
	case "s3-like":
		return o.minio.RemoveObject(ctx, o.c.Path, dir+"/"+filename, minio.RemoveObjectOptions{})
	default:
		return fmt.Errorf("operation not supported for object storage type %s", o.c.Type)
	}
}
